use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::proxy::proxy_url;
use crate::resolver::resolve_url;

#[actix_web::get("/")]
async fn resolve(query: web::Query<ResolverQuery>) -> impl Responder {
    let prime = &query.prime.unwrap_or(false);

    let resolved_url = match resolve_url(&query.url, &prime).await {
        Ok(url) => url,
        Err(_err) => {
            println!("{}", _err);
            String::from(&query.url)
        }
    };
    let response = ResolverResult {
        eurl: resolved_url,
        surl: String::from(&query.url),
    };
    return HttpResponse::Ok().json(response);
}

#[actix_web::get("/proxy")]
async fn proxy(query: web::Query<ResolverQuery>) -> impl Responder {
    if let Ok(result) = proxy_url(&query.url).await {
        if let Some(result) = result {
            let mut response = HttpResponse::Ok();
            for header in result.headers.iter() {
                if header.0.as_str().to_lowercase() != "content-encoding" {
                    response.append_header(header);
                }
            }
            return response.body(result.bytes);
        }
    }
    return HttpResponse::InternalServerError().body("");
}

#[derive(Deserialize)]
struct ResolverQuery {
    url: String,
    prime: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResolverResult {
    pub surl: String,
    pub eurl: String,
}

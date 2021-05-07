use reqwest::header;
use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use warp::{http::StatusCode, Filter};

#[tokio::main]
async fn main() {
    let resolve_route = warp::get()
        .and(warp::query())
        .and(warp::path::end())
        .and_then(resolve);

    let health_route = warp::path!("ping").map(|| StatusCode::OK);
    let routes = health_route.or(resolve_route);

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse()
        .expect("PORT must be a number");

    println!("Running on port {}", port);
    
    warp::serve(routes).run(([0, 0, 0, 0], port)).await
}

async fn resolve(query: ResolverQuery) -> Result<impl warp::Reply, warp::Rejection> {
    let resolved_url = match _resolve(&query.url).await {
        Ok(url) => url,
        Err(_err) => String::from(&query.url),
    };

    let response = ResolverResult {
        eurl: resolved_url,
        surl: query.url,
    };

    Ok(warp::reply::json(&response))
}

async fn _resolve(url: &str) -> Result<String, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Referer",
        header::HeaderValue::from_static("https://google.com"),
    );
    let client = ClientBuilder::new()
        .timeout(Duration::new(5, 0))
        .redirect(Policy::limited(4))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15")
        .default_headers(headers)
        .build()?;
    let resp = client.get(url).send().await?;
    let url = resp.url();
    let resolved_url = String::from(url.as_ref());
    Ok(resolved_url)
}

#[derive(Deserialize)]
struct ResolverQuery {
    url: String,
}

#[derive(Serialize)]
struct ResolverResult {
    surl: String,
    eurl: String,
}

use serde::{Deserialize, Serialize};
use std::env;
use warp::{http::StatusCode, Filter};

mod resolver;

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
    let resolved_url = match resolver::resolve(&query.url).await {
        Ok(url) => url,
        Err(_err) => String::from(&query.url),
    };

    let response = ResolverResult {
        eurl: resolved_url,
        surl: query.url,
    };

    println!("{:?}", &response);

    Ok(warp::reply::json(&response))
}

#[derive(Deserialize)]
struct ResolverQuery {
    url: String,
}

#[derive(Serialize, Debug)]
struct ResolverResult {
    surl: String,
    eurl: String,
}
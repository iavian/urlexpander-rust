use actix_web::{middleware, App, HttpServer};
use std::env;

pub mod resolver;
pub mod proxy;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse()
        .expect("PORT must be a number");

    let binding_interface = format!("0.0.0.0:{}", port);
    println!("Listening at {}", binding_interface);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            .service(service::resolve)
            .service(service::proxy)
    })
    .bind(binding_interface)?
    .run()
    .await
}

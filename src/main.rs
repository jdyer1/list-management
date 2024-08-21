use std::env;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use list_management::route_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
    let port_str = env::var("SERVER_PORT").expect("SERVER_PORT must be set");
    let port: u16 = port_str.parse().expect("SERVER_PORT must be a positive integer.");

    let _ = HttpServer::new(|| {
        App::new()
            .configure(route_config::config)
    }).bind((host.clone(), port)).unwrap().run().await;
    Ok(())
}

use std::env;

use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use tracing::dispatcher::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;

use list_management::helpers::tracing_subscriber;
use list_management::route_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
    let port_str = env::var("SERVER_PORT").expect("SERVER_PORT must be set");
    let port: u16 = port_str.parse().expect("SERVER_PORT must be a positive integer.");
    let log_level = env::var("LOG_LEVEL").unwrap_or("warn".to_string());

    LogTracer::init().expect("Failed to initalize the LogTracer.");
    set_global_default(tracing_subscriber(log_level, std::io::stdout).into()).expect("Failed to set subscriber");

    let _ = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .configure(route_config::config)
    }).bind((host.clone(), port)).unwrap().run().await;
    Ok(())
}

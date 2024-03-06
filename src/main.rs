mod routes;
mod controllers;
mod services;
mod common;
mod models;
mod config;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;

use crate::{config::aws_config::AwsConfig, routes::config::config, services::s3_service::S3Service};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    // building address and ip
    let port = std::env::var("PORT_API").unwrap_or("8080".to_string());
    let host = std::env::var("HOST_API").unwrap_or("127.0.0.1".to_string());
    let address = format!("{}:{}", host, port);
    println!("🚀 Server started successfully on => {}", address);

    // Instance client aws
    let aws_config = AwsConfig::init().await.unwrap();
    let client_s3 = aws_config.client_s3();
    let s3_service = web::Data::new(S3Service::new(client_s3));

    HttpServer::new(move || {
        App::new()
            .configure(config)
            .app_data(web::Data::new(services::health_service::HealthService::new()))
            .app_data(s3_service.clone())
            .wrap(Logger::default())
        })
        .bind(&address)
        .unwrap_or_else(|err| {
            panic!(
                "🔥🔥🔥 Couldn't start the server in port {}: {:?}",
                port, err
            )
        })
        .run()
        .await
}
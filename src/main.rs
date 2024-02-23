mod routes;

use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;

use crate::routes::config::config;

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

    HttpServer::new(move || {
        App::new()
            .configure(config)
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
use actix_web::{web, HttpResponse};
use log::info;

pub fn config(conf: &mut web::ServiceConfig) {
    info!("Configuring routes...");

    conf.service(
        web::scope("/api/v1")
            .default_service(web::to(|| HttpResponse::NotFound()))
    );
}
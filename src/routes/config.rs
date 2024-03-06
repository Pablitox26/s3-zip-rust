use actix_web::{web, HttpResponse};
use log::info;

use crate::routes::{health_route::health_route, s3_route::s3_route};

pub fn config(conf: &mut web::ServiceConfig) {
    info!("Configuring routes...");

    conf.service(
web::scope("/api/v1")
            .default_service(web::to(|| HttpResponse::NotFound()))
            .service(health_route())
            .service(s3_route()),
    );
}
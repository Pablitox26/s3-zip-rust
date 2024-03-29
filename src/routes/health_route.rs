use actix_web::web;
use crate::controllers::health_controller;

pub fn health_route() -> actix_web::Scope {
    web::scope("/health")
        .service(
    web::resource("")
                .route(web::get().to(health_controller::health))
        )
}
use actix_web::web;
use crate::controllers::*;

pub fn health() -> actix_web::Scope {
    web::scope("/health")
        .service(
    web::resource("")
                .route(web::get().to(health_controller::health))
        )
}
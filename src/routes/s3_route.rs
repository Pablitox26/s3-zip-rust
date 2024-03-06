use actix_web::web;
use crate::controllers::s3_controller;

pub fn s3_route() -> actix_web::Scope {
    web::scope("/s3")
        .service(
    web::resource("/compress-objects")
                .route(web::post().to(s3_controller::compress_objects))
        )
}
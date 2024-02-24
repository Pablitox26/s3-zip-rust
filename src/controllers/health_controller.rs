use actix_web::{web, HttpResponse};

use crate::services::health_service::HealthService;

pub async fn health(service: web::Data<HealthService>) -> HttpResponse {
    match service.check_health() {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
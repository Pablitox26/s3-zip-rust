use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};

use crate::services::health_service::HealthService;
use crate::common::{error::ServiceError, constants};
use crate::models::response::ResponseBody;


pub async fn health(service: web::Data<HealthService>) -> Result<HttpResponse, ServiceError> {

    match service.check_health() {
        Ok(health) => Ok(HttpResponse::Ok().json(ResponseBody {
            message: health,
            data: true,
        })),
        Err(_err) => Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string(),
        ))
    }
    
}
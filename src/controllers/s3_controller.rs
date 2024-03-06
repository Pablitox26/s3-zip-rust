use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, HttpResponse};

use crate::common::{error::ServiceError, constants};
use crate::models::file_s3::FileS3;
use crate::models::response::ResponseBody;
use crate::services::s3_service::S3Service;


pub async fn compress_objects(service: web::Data<S3Service>, files: Json<Vec<FileS3>>) -> Result<HttpResponse, ServiceError> {

    match service.compress_objects(files).await {
        Ok(objects) => Ok(HttpResponse::Ok().json(ResponseBody {
            message: constants::MESSAGE_OK.to_string(),
            data: objects,
        })),
        Err(_err) => Err(ServiceError::new(
            StatusCode::NOT_FOUND,
            constants::MESSAGE_CAN_NOT_FETCH_DATA.to_string(),
        ))
    }
    
}
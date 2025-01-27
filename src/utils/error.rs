use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use log::error;
use serde_json::json;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        error!("Error occurred: {}", self);
        match self {
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal Server Error",
                    "message": self.to_string()
                }))
            }
            AppError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad Request",
                    "message": message
                }))
            }
            AppError::Unauthorized(ref message) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized",
                    "message": message
                }))
            }
        }
    }
}
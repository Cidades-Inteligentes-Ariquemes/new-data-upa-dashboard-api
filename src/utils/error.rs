use actix_web::{error::ResponseError, HttpResponse, http::StatusCode};
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

    #[display(fmt = "Forbidden: {}", _0)]
    Forbidden(String)
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        error!("Error occurred: {}", self);
        
        let (status_code, error_type) = match self {
            AppError::InternalServerError => 
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
            AppError::BadRequest(_) => 
                (StatusCode::BAD_REQUEST, "Bad Request"),
            AppError::Unauthorized(_) => 
                (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden(_) => 
                (StatusCode::FORBIDDEN, "Forbidden")
        };

        HttpResponse::build(status_code)
            .json(json!({
                "error": error_type,
                "message": self.to_string(),
                "status_code": status_code.as_u16()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN
        }
    }
}
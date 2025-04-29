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
    Forbidden(String),

    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),

    #[display(fmt = "Database Error: {}", _0)]
    DatabaseError(String),

    #[display(fmt = "Data Processing Error: {}", _0)]
    DataProcessingError(String),

    //InvalidMethodError
    #[display(fmt = "Invalid Method Error: {}", _0)]
    InvalidMethodError(String),
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
                (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound(_) =>
                (StatusCode::NOT_FOUND, "Not Found"),
            AppError::DatabaseError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            AppError::DataProcessingError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "Data Processing Error"),
            AppError::InvalidMethodError(_) =>
                (StatusCode::METHOD_NOT_ALLOWED, "Invalid Method Error"),
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
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DataProcessingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidMethodError(_) => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}
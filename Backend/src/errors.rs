use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            error: self.to_string(),
        };

        match self {
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(body),
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized().json(body),
            ApiError::NotFound(_) => HttpResponse::NotFound().json(body),
            ApiError::Internal => HttpResponse::InternalServerError().json(body),
        }
    }
}

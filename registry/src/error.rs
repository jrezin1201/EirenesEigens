use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::models::ErrorResponse;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
    InternalError(String),
    InternalServerError(String),
    ValidationError(String),
    RateLimitExceeded,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                format!("Database error: {}", err),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg,
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_server_error",
                msg,
            ),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
                "Rate limit exceeded. Please try again later.".to_string(),
            ),
        };

        let error_response = ErrorResponse::new(error_code, message);
        (status, Json(error_response)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::InternalError(format!("Password hashing error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized(format!("Invalid token: {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::InternalError(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON error: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;

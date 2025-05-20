use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Custom API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

/// Implementation to convert API errors into HTTP responses
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(ref err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            ApiError::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            ApiError::InvalidInput(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            ApiError::Blockchain(ref message) => (StatusCode::BAD_GATEWAY, message.clone()),
            ApiError::Internal(ref message) => (StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
            ApiError::Unauthorized(ref message) => (StatusCode::UNAUTHORIZED, message.clone()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

/// For convenience, implement From for anyhow::Error
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>; 
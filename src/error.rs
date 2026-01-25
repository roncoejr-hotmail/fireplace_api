use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid command type")]
    InvalidCommand,

    #[error("Invalid action")]
    InvalidAction,

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("GPIO error: {0}")]
    GpioError(String),

    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::InvalidCommand => (
                StatusCode::BAD_REQUEST,
                "Invalid command type. Expected ''toggle''".to_string(),
            ),
            ApiError::InvalidAction => (
                StatusCode::BAD_REQUEST,
                "Invalid action. Expected ''ON'' or ''OFF''".to_string(),
            ),
            ApiError::InvalidPin => (
                StatusCode::BAD_REQUEST,
                "Invalid GPIO pin".to_string(),
            ),
            ApiError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
            ApiError::GpioError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": true,
            "message": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;

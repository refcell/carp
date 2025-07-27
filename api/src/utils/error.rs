use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "ValidationError" => StatusCode::BAD_REQUEST,
            "AuthenticationError" => StatusCode::UNAUTHORIZED,
            "AuthorizationError" => StatusCode::FORBIDDEN,
            "NotFoundError" => StatusCode::NOT_FOUND,
            "ConflictError" => StatusCode::CONFLICT,
            "RateLimitError" => StatusCode::TOO_MANY_REQUESTS,
            "PayloadTooLarge" => StatusCode::PAYLOAD_TOO_LARGE,
            "UnsupportedMediaType" => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("ValidationError", message)
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::new("AuthenticationError", message)
    }

    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::new("AuthorizationError", message)
    }

    pub fn not_found_error(message: impl Into<String>) -> Self {
        Self::new("NotFoundError", message)
    }

    pub fn conflict_error(message: impl Into<String>) -> Self {
        Self::new("ConflictError", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("InternalError", message)
    }

    pub fn rate_limit_error() -> Self {
        Self::new("RateLimitError", "Too many requests")
    }

    pub fn payload_too_large() -> Self {
        Self::new("PayloadTooLarge", "Request payload too large")
    }

    pub fn unsupported_media_type() -> Self {
        Self::new("UnsupportedMediaType", "Unsupported media type")
    }
}

// From implementations for common error types
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Internal error: {}", err);
        Self::internal_error("Internal server error")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::validation_error(format!("JSON parsing error: {}", err))
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("HTTP client error: {}", err);
        Self::internal_error("External service error")
    }
}

impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        Self::validation_error(format!("Invalid UUID: {}", err))
    }
}

// PostgreSQL client errors are handled directly in handlers for now
// impl From<postgrest::Error> for ApiError {
//     fn from(err: postgrest::Error) -> Self {
//         tracing::error!("Database error: {}", err);
//         Self::internal_error("Database operation failed")
//     }
// }
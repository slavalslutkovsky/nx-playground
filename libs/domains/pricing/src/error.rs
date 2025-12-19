use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Result type for pricing operations
pub type PricingResult<T> = Result<T, PricingError>;

/// Errors that can occur in the pricing domain
#[derive(Debug, Error)]
pub enum PricingError {
    /// Price not found
    #[error("Price not found: {0}")]
    NotFound(String),

    /// Duplicate price entry
    #[error("Duplicate price entry: {0}")]
    Duplicate(String),

    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Provider error
    #[error("Provider error: {0}")]
    Provider(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for PricingError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            PricingError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PricingError::Duplicate(_) => (StatusCode::CONFLICT, self.to_string()),
            PricingError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PricingError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            PricingError::Provider(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            PricingError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<PricingError> for tonic::Status {
    fn from(err: PricingError) -> Self {
        match err {
            PricingError::NotFound(msg) => tonic::Status::not_found(msg),
            PricingError::Duplicate(msg) => tonic::Status::already_exists(msg),
            PricingError::InvalidInput(msg) => tonic::Status::invalid_argument(msg),
            PricingError::Database(e) => tonic::Status::internal(e.to_string()),
            PricingError::Provider(msg) => tonic::Status::unavailable(msg),
            PricingError::Internal(msg) => tonic::Status::internal(msg),
        }
    }
}

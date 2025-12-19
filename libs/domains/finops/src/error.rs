use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Result type for finops operations
pub type FinopsResult<T> = Result<T, FinopsError>;

/// Errors that can occur in the finops domain
#[derive(Debug, Error)]
pub enum FinopsError {
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Cloud account not found
    #[error("Cloud account not found: {0}")]
    AccountNotFound(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Agent error
    #[error("Agent error: {0}")]
    Agent(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Cloud provider error
    #[error("Cloud provider error: {0}")]
    CloudProvider(String),

    /// Action not approved
    #[error("Action requires approval")]
    NotApproved,

    /// Action not confirmed
    #[error("Action requires confirmation")]
    NotConfirmed,

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Pricing domain error
    #[error("Pricing error: {0}")]
    Pricing(#[from] domain_pricing::PricingError),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for FinopsError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            FinopsError::SessionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FinopsError::AccountNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FinopsError::ResourceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FinopsError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            FinopsError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            FinopsError::Agent(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FinopsError::ToolExecution(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FinopsError::CloudProvider(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            FinopsError::NotApproved => (StatusCode::FORBIDDEN, self.to_string()),
            FinopsError::NotConfirmed => (StatusCode::BAD_REQUEST, self.to_string()),
            FinopsError::Timeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            FinopsError::Pricing(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FinopsError::Internal(_) => (
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

use axum::response::{IntoResponse, Response};
use axum_helpers::AppError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ItemError {
    #[error("Item not found: {0}")]
    NotFound(Uuid),

    #[error("Item with name '{0}' already exists")]
    DuplicateName(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ItemResult<T> = Result<T, ItemError>;

/// Convert ItemError to AppError for standardized error responses
impl From<ItemError> for AppError {
    fn from(err: ItemError) -> Self {
        match err {
            ItemError::NotFound(id) => AppError::NotFound(format!("Item {} not found", id)),
            ItemError::DuplicateName(name) => {
                AppError::Conflict(format!("Item with name '{}' already exists", name))
            }
            ItemError::Validation(msg) => AppError::BadRequest(msg),
            ItemError::Database(msg) => AppError::InternalServerError(msg),
            ItemError::Internal(msg) => AppError::InternalServerError(msg),
        }
    }
}

impl IntoResponse for ItemError {
    fn into_response(self) -> Response {
        // Convert to AppError for the standardized error response format
        let app_error: AppError = self.into();
        app_error.into_response()
    }
}

impl From<mongodb::error::Error> for ItemError {
    fn from(err: mongodb::error::Error) -> Self {
        ItemError::Database(err.to_string())
    }
}

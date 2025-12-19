use axum::response::{IntoResponse, Response};
use axum_helpers::AppError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(Uuid),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("Stream error: {0}")]
    Stream(String),
}

pub type TaskResult<T> = Result<T, TaskError>;

/// Convert TaskError to AppError for standardized error responses
impl From<TaskError> for AppError {
    fn from(err: TaskError) -> Self {
        match err {
            TaskError::NotFound(id) => AppError::NotFound(format!("Task {} not found", id)),
            TaskError::Validation(msg) => AppError::BadRequest(msg),
            TaskError::Internal(msg) => AppError::InternalServerError(msg),
            TaskError::Database(msg) => {
                AppError::InternalServerError(format!("Database error: {}", msg))
            }
            TaskError::Timeout(msg) => AppError::ServiceUnavailable(format!("Timeout: {}", msg)),
            TaskError::Stream(msg) => {
                AppError::InternalServerError(format!("Stream error: {}", msg))
            }
        }
    }
}

impl IntoResponse for TaskError {
    fn into_response(self) -> Response {
        // Convert to AppError for the standardized error response format
        let app_error: AppError = self.into();
        app_error.into_response()
    }
}

/// Implement From for sea_orm::DbErr
impl From<sea_orm::DbErr> for TaskError {
    fn from(err: sea_orm::DbErr) -> Self {
        TaskError::Database(err.to_string())
    }
}

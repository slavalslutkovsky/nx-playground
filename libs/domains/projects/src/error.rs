use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Project not found: {0}")]
    NotFound(Uuid),

    #[error("Project with name '{0}' already exists for this user")]
    DuplicateName(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Unauthorized access to project {0}")]
    Unauthorized(Uuid),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ProjectResult<T> = Result<T, ProjectError>;

impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            ProjectError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("Project {} not found", id),
            ),
            ProjectError::DuplicateName(name) => (
                StatusCode::CONFLICT,
                "duplicate",
                format!("Project with name '{}' already exists", name),
            ),
            ProjectError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg.clone())
            }
            ProjectError::Unauthorized(id) => (
                StatusCode::FORBIDDEN,
                "unauthorized",
                format!("Access denied to project {}", id),
            ),
            ProjectError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": {
                    "type": error_type,
                    "message": message
                }
            })),
        )
            .into_response()
    }
}

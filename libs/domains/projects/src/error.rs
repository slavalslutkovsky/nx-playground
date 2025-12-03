use axum::response::{IntoResponse, Response};
use axum_helpers::AppError;
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

/// Convert ProjectError to AppError for standardized error responses
impl From<ProjectError> for AppError {
    fn from(err: ProjectError) -> Self {
        match err {
            ProjectError::NotFound(id) => AppError::NotFound(format!("Project {} not found", id)),
            ProjectError::DuplicateName(name) => {
                AppError::Conflict(format!("Project with name '{}' already exists", name))
            }
            ProjectError::Validation(msg) => AppError::BadRequest(msg),
            ProjectError::Unauthorized(id) => {
                AppError::Forbidden(format!("Access denied to project {}", id))
            }
            ProjectError::Internal(msg) => AppError::InternalServerError(msg),
        }
    }
}

impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        // Convert to AppError for standardized error response format
        let app_error: AppError = self.into();
        app_error.into_response()
    }
}

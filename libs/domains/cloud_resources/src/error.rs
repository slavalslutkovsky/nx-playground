use axum_helpers::{AppError, impl_into_response_via_app_error};
use thiserror::Error;
use uuid::Uuid;

pub type CloudResourceResult<T> = Result<T, CloudResourceError>;

#[derive(Debug, Error)]
pub enum CloudResourceError {
    #[error("Cloud resource not found: {0}")]
    NotFound(Uuid),

    #[error("Project not found: {0}")]
    ProjectNotFound(Uuid),

    #[error("Duplicate cloud resource name: {0}")]
    DuplicateName(String),

    #[error("Invalid cloud resource status transition: {0}")]
    InvalidStatusTransition(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CloudResourceError> for AppError {
    fn from(err: CloudResourceError) -> Self {
        match err {
            CloudResourceError::NotFound(id) => {
                AppError::NotFound(format!("Cloud resource {} not found", id))
            }
            CloudResourceError::ProjectNotFound(id) => {
                AppError::NotFound(format!("Project {} not found", id))
            }
            CloudResourceError::DuplicateName(name) => AppError::Conflict(format!(
                "Cloud resource with name '{}' already exists in this project",
                name
            )),
            CloudResourceError::InvalidStatusTransition(msg) => AppError::BadRequest(msg),
            CloudResourceError::Internal(msg) => AppError::InternalServerError(msg),
        }
    }
}

impl_into_response_via_app_error!(CloudResourceError);

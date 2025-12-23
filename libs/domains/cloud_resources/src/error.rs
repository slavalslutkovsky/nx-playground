use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use uuid::Uuid;

pub type CloudResourceResult<T> = Result<T, CloudResourceError>;

#[derive(Debug, thiserror::Error)]
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

impl IntoResponse for CloudResourceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::NotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("Cloud resource {} not found", id),
            ),
            Self::ProjectNotFound(id) => {
                (StatusCode::NOT_FOUND, format!("Project {} not found", id))
            }
            Self::DuplicateName(name) => (
                StatusCode::CONFLICT,
                format!(
                    "Cloud resource with name '{}' already exists in this project",
                    name
                ),
            ),
            Self::InvalidStatusTransition(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

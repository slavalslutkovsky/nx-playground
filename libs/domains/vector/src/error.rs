use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum VectorError {
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Vector not found: {0}")]
    VectorNotFound(Uuid),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Qdrant error: {0}")]
    Qdrant(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type VectorResult<T> = Result<T, VectorError>;

impl From<qdrant_client::QdrantError> for VectorError {
    fn from(err: qdrant_client::QdrantError) -> Self {
        VectorError::Qdrant(err.to_string())
    }
}

impl From<reqwest::Error> for VectorError {
    fn from(err: reqwest::Error) -> Self {
        VectorError::Embedding(err.to_string())
    }
}

impl From<serde_json::Error> for VectorError {
    fn from(err: serde_json::Error) -> Self {
        VectorError::Internal(format!("JSON error: {}", err))
    }
}

impl From<tonic::Status> for VectorError {
    fn from(status: tonic::Status) -> Self {
        VectorError::Internal(format!("gRPC error: {}", status.message()))
    }
}

impl From<VectorError> for tonic::Status {
    fn from(err: VectorError) -> Self {
        match err {
            VectorError::CollectionNotFound(name) => {
                tonic::Status::not_found(format!("Collection not found: {}", name))
            }
            VectorError::VectorNotFound(id) => {
                tonic::Status::not_found(format!("Vector not found: {}", id))
            }
            VectorError::Validation(msg) => tonic::Status::invalid_argument(msg),
            VectorError::Qdrant(msg) => tonic::Status::internal(format!("Qdrant error: {}", msg)),
            VectorError::Embedding(msg) => {
                tonic::Status::internal(format!("Embedding error: {}", msg))
            }
            VectorError::Config(msg) => {
                tonic::Status::failed_precondition(format!("Config error: {}", msg))
            }
            VectorError::Internal(msg) => tonic::Status::internal(msg),
        }
    }
}

// Axum IntoResponse implementation for HTTP handlers
impl axum::response::IntoResponse for VectorError {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;
        use axum::http::StatusCode;

        let (status, message) = match &self {
            VectorError::CollectionNotFound(name) => (
                StatusCode::NOT_FOUND,
                format!("Collection not found: {}", name),
            ),
            VectorError::VectorNotFound(id) => {
                (StatusCode::NOT_FOUND, format!("Vector not found: {}", id))
            }
            VectorError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            VectorError::Qdrant(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Qdrant error: {}", msg),
            ),
            VectorError::Embedding(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding error: {}", msg),
            ),
            VectorError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Config error: {}", msg),
            ),
            VectorError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = serde_json::json!({
            "error": message,
            "code": status.as_u16()
        });

        (status, Json(body)).into_response()
    }
}

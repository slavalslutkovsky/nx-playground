//! Event domain error types

use axum_helpers::AppError;
use std::fmt;

/// Result type for event operations
pub type Result<T> = std::result::Result<T, EventError>;

/// Event domain errors
#[derive(Debug)]
pub enum EventError {
    /// Event not found
    NotFound { id: String },

    /// Validation error
    Validation { message: String },

    /// MongoDB error
    Database {
        message: String,
        source: Option<mongodb::error::Error>,
    },

    /// InfluxDB error
    InfluxDb { message: String },

    /// Dapr error
    Dapr { message: String },

    /// Serialization error
    Serialization { message: String },

    /// Internal error
    Internal { message: String },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "Event not found: {}", id),
            Self::Validation { message } => write!(f, "Validation error: {}", message),
            Self::Database { message, .. } => write!(f, "Database error: {}", message),
            Self::InfluxDb { message } => write!(f, "InfluxDB error: {}", message),
            Self::Dapr { message } => write!(f, "Dapr error: {}", message),
            Self::Serialization { message } => write!(f, "Serialization error: {}", message),
            Self::Internal { message } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for EventError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database {
                source: Some(e), ..
            } => Some(e),
            _ => None,
        }
    }
}

impl From<mongodb::error::Error> for EventError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::Database {
            message: err.to_string(),
            source: Some(err),
        }
    }
}

impl From<mongodb::bson::oid::Error> for EventError {
    fn from(err: mongodb::bson::oid::Error) -> Self {
        Self::Database {
            message: format!("Invalid ObjectId: {}", err),
            source: None,
        }
    }
}

impl From<mongodb::bson::ser::Error> for EventError {
    fn from(err: mongodb::bson::ser::Error) -> Self {
        Self::Database {
            message: format!("BSON serialization error: {}", err),
            source: None,
        }
    }
}

impl From<mongodb::bson::de::Error> for EventError {
    fn from(err: mongodb::bson::de::Error) -> Self {
        Self::Database {
            message: format!("BSON deserialization error: {}", err),
            source: None,
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<validator::ValidationErrors> for EventError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for EventError {
    fn from(err: reqwest::Error) -> Self {
        Self::Dapr {
            message: err.to_string(),
        }
    }
}

// Convert to axum_helpers::AppError for HTTP responses
impl From<EventError> for AppError {
    fn from(err: EventError) -> Self {
        match err {
            EventError::NotFound { id } => AppError::NotFound(format!("Event not found: {}", id)),
            EventError::Validation { message } => AppError::BadRequest(message),
            EventError::Database { message, .. } => AppError::InternalServerError(message),
            EventError::InfluxDb { message } => AppError::InternalServerError(message),
            EventError::Dapr { message } => {
                AppError::InternalServerError(format!("Dapr error: {}", message))
            }
            EventError::Serialization { message } => AppError::InternalServerError(message),
            EventError::Internal { message } => AppError::InternalServerError(message),
        }
    }
}

impl axum::response::IntoResponse for EventError {
    fn into_response(self) -> axum::response::Response {
        let app_error: AppError = self.into();
        app_error.into_response()
    }
}

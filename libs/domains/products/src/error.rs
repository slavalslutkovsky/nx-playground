use axum::response::{IntoResponse, Response};
use axum_helpers::AppError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProductError {
    #[error("Product not found: {0}")]
    NotFound(Uuid),

    #[error("Product with SKU '{0}' already exists")]
    DuplicateSku(String),

    #[error("Product with name '{0}' already exists")]
    DuplicateName(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Insufficient stock: available {available}, requested {requested}")]
    InsufficientStock { available: i32, requested: i32 },

    #[error("Reservation not found: {0}")]
    ReservationNotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Dapr error: {0}")]
    Dapr(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ProductResult<T> = Result<T, ProductError>;

/// Convert ProductError to AppError for standardized error responses
impl From<ProductError> for AppError {
    fn from(err: ProductError) -> Self {
        match err {
            ProductError::NotFound(id) => AppError::NotFound(format!("Product {} not found", id)),
            ProductError::DuplicateSku(sku) => {
                AppError::Conflict(format!("Product with SKU '{}' already exists", sku))
            }
            ProductError::DuplicateName(name) => {
                AppError::Conflict(format!("Product with name '{}' already exists", name))
            }
            ProductError::Validation(msg) => AppError::BadRequest(msg),
            ProductError::InsufficientStock {
                available,
                requested,
            } => AppError::BadRequest(format!(
                "Insufficient stock: {} available, {} requested",
                available, requested
            )),
            ProductError::ReservationNotFound(id) => {
                AppError::NotFound(format!("Reservation {} not found", id))
            }
            ProductError::Database(msg) => AppError::InternalServerError(msg),
            ProductError::Cache(msg) => AppError::InternalServerError(msg),
            ProductError::Dapr(msg) => AppError::InternalServerError(msg),
            ProductError::Internal(msg) => AppError::InternalServerError(msg),
        }
    }
}

impl IntoResponse for ProductError {
    fn into_response(self) -> Response {
        let app_error: AppError = self.into();
        app_error.into_response()
    }
}

impl From<mongodb::error::Error> for ProductError {
    fn from(err: mongodb::error::Error) -> Self {
        ProductError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for ProductError {
    fn from(err: redis::RedisError) -> Self {
        ProductError::Cache(err.to_string())
    }
}

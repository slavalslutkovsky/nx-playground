use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use super::ErrorResponse;

/// Handler for 404 Not Found errors.
///
/// This can be used as a fallback handler in your router.
pub async fn not_found() -> Response {
    let body = Json(ErrorResponse {
        error: "NotFound".to_string(),
        message: "The requested resource was not found".to_string(),
        details: None,
    });

    (StatusCode::NOT_FOUND, body).into_response()
}

/// Handler for 405 Method Not Allowed errors.
pub async fn method_not_allowed() -> Response {
    let body = Json(ErrorResponse {
        error: "MethodNotAllowed".to_string(),
        message: "The HTTP method is not allowed for this resource".to_string(),
        details: None,
    });

    (StatusCode::METHOD_NOT_ALLOWED, body).into_response()
}

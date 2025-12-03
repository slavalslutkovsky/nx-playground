//! Reusable OpenAPI response types for consistent API documentation.

use super::ErrorResponse;
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToResponse;

/// Standard error messages for consistent API responses
pub mod messages {
    pub const INTERNAL_ERROR: &str = "An internal server error occurred";
    pub const VALIDATION_FAILED: &str = "Request validation failed";
    pub const INVALID_UUID: &str = "Invalid UUID format";
    pub const NOT_FOUND_RESOURCE: &str = "Resource not found";
    pub const UNAUTHORIZED: &str = "Authentication required";
    pub const FORBIDDEN: &str = "Access forbidden";

    // Error codes for client parsing
    pub const CODE_INTERNAL: &str = "INTERNAL_ERROR";
    pub const CODE_VALIDATION: &str = "VALIDATION_ERROR";
    pub const CODE_UUID: &str = "INVALID_UUID";
    pub const CODE_NOT_FOUND: &str = "NOT_FOUND";
    pub const CODE_UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const CODE_FORBIDDEN: &str = "FORBIDDEN";
}

#[derive(ToResponse)]
#[response(
    description = "Internal Server Error",
    content_type = "application/json",
    example = json!({
        "error": "InternalServerError",
        "message": "An internal server error occurred",
        "details": null
    })
)]
pub struct InternalServerErrorResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Bad Request - Validation Error",
    content_type = "application/json",
    example = json!({
        "error": "BadRequest",
        "message": "Request validation failed",
        "details": {
            "title": [{
                "code": "length",
                "message": "length is less than 3",
                "params": {"min": 3, "value": "ab"}
            }]
        }
    })
)]
pub struct BadRequestValidationResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Bad Request - Invalid UUID",
    content_type = "application/json",
    example = json!({
        "error": "BadRequest",
        "message": "Invalid UUID format",
        "details": null
    })
)]
pub struct BadRequestUuidResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Resource not found",
    content_type = "application/json",
    example = json!({
        "error": "NotFound",
        "message": "Resource not found",
        "details": null
    })
)]
pub struct NotFoundResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Unauthorized - Authentication required",
    content_type = "application/json",
    example = json!({
        "error": "Unauthorized",
        "message": "Authentication required",
        "details": null
    })
)]
pub struct UnauthorizedResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Forbidden - Insufficient permissions",
    content_type = "application/json",
    example = json!({
        "error": "Forbidden",
        "message": "Access forbidden",
        "details": null
    })
)]
pub struct ForbiddenResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Conflict - Resource already exists",
    content_type = "application/json",
    example = json!({
        "error": "Conflict",
        "message": "Resource already exists",
        "details": null
    })
)]
pub struct ConflictResponse(pub ErrorResponse);

#[derive(ToResponse)]
#[response(
    description = "Service Unavailable",
    content_type = "application/json",
    example = json!({
        "error": "ServiceUnavailable",
        "message": "Service is temporarily unavailable",
        "details": null
    })
)]
pub struct ServiceUnavailableResponse(pub ErrorResponse);

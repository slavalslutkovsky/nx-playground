//! Reusable OpenAPI response types for consistent API documentation.
//!
//! This module provides standardized response types that can be referenced in
//! `#[utoipa::path]` endpoint definitions. Each response type includes:
//! - HTTP status code mapping
//! - JSON schema definition
//! - Example response payload
//!
//! # Usage
//!
//! Reference these types in your OpenAPI path definitions:
//!
//! ```rust,ignore
//! #[utoipa::path(
//!     get,
//!     path = "/resource/{id}",
//!     responses(
//!         (status = 200, description = "Success", body = Resource),
//!         (status = 404, response = NotFoundResponse),
//!         (status = 500, response = InternalServerErrorResponse)
//!     )
//! )]
//! ```
//!
//! Also register them in your OpenAPI documentation:
//!
//! ```rust,ignore
//! #[derive(OpenApi)]
//! #[openapi(
//!     components(
//!         responses(
//!             NotFoundResponse,
//!             BadRequestValidationResponse,
//!             InternalServerErrorResponse
//!         )
//!     )
//! )]
//! ```

use super::ErrorResponse;
use utoipa::ToResponse;

// Note: json! macro is used in #[response] examples but compiler can't detect it
#[allow(unused_imports)]
use serde_json::json;

/// OpenAPI response type for 500 Internal Server Error.
///
/// Use this response when an unexpected server-side error occurs that prevents
/// the request from being fulfilled. Common scenarios include:
///
/// - Database connection failures
/// - Configuration errors
/// - Unexpected panics or exceptions
/// - Resource exhaustion
/// - Third-party service failures
///
/// The actual error details are logged server-side but not exposed to clients
/// for security reasons.
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     get,
///     path = "/resource",
///     responses(
///         (status = 200, description = "Success", body = Resource),
///         (status = 500, response = InternalServerErrorResponse)
///     )
/// )]
/// async fn get_resource() -> Result<Json<Resource>, AppError> {
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Internal Server Error",
    content_type = "application/json",
    example = json!({
        "code": 1005,
        "error": "INTERNAL_ERROR",
        "message": "An internal server error occurred",
        "details": null
    })
)]
pub struct InternalServerErrorResponse(pub ErrorResponse);

/// OpenAPI response type for 400 Bad Request - Validation Error.
///
/// Use this response when request body validation fails. This typically occurs when
/// using the `ValidatedJson` extractor with the `validator` crate.
///
/// The `details` field contains structured validation errors with:
/// - Field name as the key
/// - Array of validation failures for that field
/// - Each failure includes: validation code, message, and relevant parameters
///
/// # Common Validation Scenarios
///
/// - String length constraints (min, max)
/// - Email format validation
/// - Required fields missing
/// - Numeric range violations
/// - Custom validation rules
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     post,
///     path = "/resource",
///     request_body = CreateResourceRequest,
///     responses(
///         (status = 201, description = "Created", body = Resource),
///         (status = 400, response = BadRequestValidationResponse)
///     )
/// )]
/// async fn create_resource(
///     ValidatedJson(input): ValidatedJson<CreateResourceRequest>
/// ) -> Result<Json<Resource>, AppError> {
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Bad Request - Validation Error",
    content_type = "application/json",
    example = json!({
        "code": 1001,
        "error": "VALIDATION_ERROR",
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

/// OpenAPI response type for 400 Bad Request - Invalid UUID.
///
/// Use this response when a path or query parameter expected to be a UUID
/// cannot be parsed. This typically occurs when using the `UuidPath` extractor.
///
/// # Common Scenarios
///
/// - Malformed UUID string (wrong length, invalid characters)
/// - Using non-UUID values in UUID path parameters
/// - Invalid UUID format in query parameters
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     get,
///     path = "/resource/{id}",
///     params(
///         ("id" = Uuid, Path, description = "Resource ID")
///     ),
///     responses(
///         (status = 200, description = "Found", body = Resource),
///         (status = 400, response = BadRequestUuidResponse),
///         (status = 404, response = NotFoundResponse)
///     )
/// )]
/// async fn get_resource(
///     UuidPath(id): UuidPath
/// ) -> Result<Json<Resource>, AppError> {
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Bad Request - Invalid UUID",
    content_type = "application/json",
    example = json!({
        "code": 1002,
        "error": "INVALID_UUID",
        "message": "Invalid UUID format",
        "details": null
    })
)]
pub struct BadRequestUuidResponse(pub ErrorResponse);

/// OpenAPI response type for 404 Not Found.
///
/// Use this response when the requested resource does not exist in the system.
/// This is typically returned when:
///
/// - A database query returns no results for a given ID
/// - A user attempts to access a resource that was deleted
/// - An endpoint path exists but the specific resource identifier is invalid
///
/// # Best Practices
///
/// - Return 404 for missing resources, not 400
/// - Don't expose whether a resource exists for security reasons (unless appropriate)
/// - Use consistent error messages
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     get,
///     path = "/resource/{id}",
///     responses(
///         (status = 200, description = "Found", body = Resource),
///         (status = 404, response = NotFoundResponse)
///     )
/// )]
/// async fn get_resource(id: Uuid) -> Result<Json<Resource>, AppError> {
///     let resource = service.find(id).await
///         .ok_or_else(|| AppError::NotFound(format!("Resource {} not found", id)))?;
///     Ok(Json(resource))
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Resource not found",
    content_type = "application/json",
    example = json!({
        "code": 1004,
        "error": "NOT_FOUND",
        "message": "Resource not found",
        "details": null
    })
)]
pub struct NotFoundResponse(pub ErrorResponse);

/// OpenAPI response type for 401 Unauthorized.
///
/// Use this response when authentication is required but not provided, or when
/// the provided authentication credentials are invalid.
///
/// # Common Scenarios
///
/// - Missing authentication token (JWT, session cookie, API key)
/// - Expired authentication token
/// - Invalid credentials (wrong username/password)
/// - Malformed authentication header
///
/// # HTTP 401 vs 403
///
/// - **401 Unauthorized**: Authentication is missing or invalid. The client should
///   authenticate and retry.
/// - **403 Forbidden**: Authentication succeeded but the user lacks permission.
///   Retrying with the same credentials will not help.
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     get,
///     path = "/protected/resource",
///     responses(
///         (status = 200, description = "Success", body = Resource),
///         (status = 401, response = UnauthorizedResponse)
///     ),
///     security(("bearer_auth" = []))
/// )]
/// async fn protected_resource(
///     auth: AuthenticatedUser
/// ) -> Result<Json<Resource>, AppError> {
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Unauthorized - Authentication required",
    content_type = "application/json",
    example = json!({
        "code": 1006,
        "error": "UNAUTHORIZED",
        "message": "Authentication required",
        "details": null
    })
)]
pub struct UnauthorizedResponse(pub ErrorResponse);

/// OpenAPI response type for 403 Forbidden.
///
/// Use this response when the authenticated user does not have sufficient permissions
/// to perform the requested action. The user is authenticated (unlike 401) but lacks
/// the necessary authorization.
///
/// # Common Scenarios
///
/// - User attempting to access another user's resources
/// - Role-based access control (RBAC) violations
/// - Tenant isolation enforcement
/// - Rate limiting or quota exceeded
/// - Disabled or suspended account
///
/// # HTTP 401 vs 403
///
/// - **401 Unauthorized**: No valid authentication provided
/// - **403 Forbidden**: Valid authentication but insufficient permissions
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     delete,
///     path = "/resource/{id}",
///     responses(
///         (status = 204, description = "Deleted"),
///         (status = 403, response = ForbiddenResponse),
///         (status = 404, response = NotFoundResponse)
///     )
/// )]
/// async fn delete_resource(
///     auth: AuthenticatedUser,
///     id: Uuid
/// ) -> Result<StatusCode, AppError> {
///     if !auth.can_delete(id) {
///         return Err(AppError::Forbidden("Insufficient permissions".to_string()));
///     }
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Forbidden - Insufficient permissions",
    content_type = "application/json",
    example = json!({
        "code": 1007,
        "error": "FORBIDDEN",
        "message": "Access forbidden",
        "details": null
    })
)]
pub struct ForbiddenResponse(pub ErrorResponse);

/// OpenAPI response type for 409 Conflict.
///
/// Use this response when the request cannot be completed due to a conflict with
/// the current state of the resource. This is commonly used for:
///
/// # Common Scenarios
///
/// - **Duplicate Resources**: Attempting to create a resource that already exists
///   (e.g., username, email, unique name)
/// - **Concurrent Updates**: Version conflicts in optimistic locking scenarios
/// - **State Conflicts**: Invalid state transitions (e.g., deleting an active resource
///   that must be suspended first)
/// - **Unique Constraint Violations**: Database unique constraint violations
///
/// # Best Practices
///
/// - Provide helpful error messages indicating what caused the conflict
/// - Include the conflicting field name when possible
/// - Suggest corrective actions to the client
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     post,
///     path = "/projects",
///     request_body = CreateProject,
///     responses(
///         (status = 201, description = "Created", body = Project),
///         (status = 409, response = ConflictResponse)
///     )
/// )]
/// async fn create_project(
///     input: ValidatedJson<CreateProject>
/// ) -> Result<Json<Project>, AppError> {
///     // Check for duplicate name
///     if service.exists_by_name(&input.name).await? {
///         return Err(AppError::Conflict(
///             format!("Project with name '{}' already exists", input.name)
///         ));
///     }
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Conflict - Resource already exists",
    content_type = "application/json",
    example = json!({
        "code": 1008,
        "error": "CONFLICT",
        "message": "Resource already exists",
        "details": null
    })
)]
pub struct ConflictResponse(pub ErrorResponse);

/// OpenAPI response type for 503 Service Unavailable.
///
/// Use this response when the service is temporarily unable to handle requests.
/// This indicates a temporary condition that should resolve itself.
///
/// # Common Scenarios
///
/// - Database connection pool exhausted
/// - Dependency service is down (circuit breaker open)
/// - Server is overloaded or throttling requests
/// - Maintenance mode or graceful shutdown in progress
/// - Health check failures
///
/// # Client Behavior
///
/// Clients receiving 503 should:
/// - Retry the request after a delay
/// - Check the `Retry-After` header if provided
/// - Use exponential backoff for retries
/// - Consider circuit breaker patterns
///
/// # Example Usage
///
/// ```rust,ignore
/// #[utoipa::path(
///     get,
///     path = "/resource",
///     responses(
///         (status = 200, description = "Success", body = Vec<Resource>),
///         (status = 503, response = ServiceUnavailableResponse)
///     )
/// )]
/// async fn list_resources(
///     State(db): State<DatabasePool>
/// ) -> Result<Json<Vec<Resource>>, AppError> {
///     // Connection pool timeout returns ServiceUnavailable
///     let conn = db.acquire().await
///         .map_err(|e| AppError::ServiceUnavailable(e.to_string()))?;
///     // ... implementation
/// }
/// ```
#[derive(ToResponse)]
#[response(
    description = "Service Unavailable",
    content_type = "application/json",
    example = json!({
        "code": 1011,
        "error": "SERVICE_UNAVAILABLE",
        "message": "Service is temporarily unavailable",
        "details": null
    })
)]
pub struct ServiceUnavailableResponse(pub ErrorResponse);

//! JSON extractor with automatic validation using the validator crate.

use crate::errors::{messages, ErrorResponse};
use axum::{
    extract::{FromRequest, Json, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// JSON extractor with automatic validation.
///
/// Validates the request body using the `validator` crate's `Validate` trait.
/// Returns structured validation errors if validation fails.
///
/// # Example
/// ```ignore
/// use axum::Router;
/// use axum::routing::post;
/// use axum_helpers::extractors::ValidatedJson;
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(length(min = 3, max = 50))]
///     username: String,
///     #[validate(email)]
///     email: String,
/// }
///
/// async fn create_user(ValidatedJson(payload): ValidatedJson<CreateUser>) -> String {
///     format!("Creating user: {}", payload.username)
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// ```
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| e.into_response())?;

        data.validate().map_err(|e| {
            // Convert validator errors to structured JSON
            let details = e
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error_messages: Vec<serde_json::Value> = errors
                        .iter()
                        .map(|err| {
                            serde_json::json!({
                                "code": err.code,
                                "message": err.message,
                                "params": err.params,
                            })
                        })
                        .collect();
                    (field.to_string(), serde_json::json!(error_messages))
                })
                .collect::<serde_json::Map<_, _>>();

            let error_response = ErrorResponse {
                error: "BadRequest".to_string(),
                message: "Request validation failed".to_string(),
                details: Some(serde_json::Value::Object(details)),
                code: Some(messages::CODE_VALIDATION),
            };

            (StatusCode::BAD_REQUEST, axum::Json(error_response)).into_response()
        })?;

        Ok(ValidatedJson(data))
    }
}

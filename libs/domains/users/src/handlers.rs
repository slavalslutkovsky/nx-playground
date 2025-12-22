use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_helpers::ValidatedJson;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::UserResult;
use crate::models::{CreateUser, LoginRequest, UpdateUser, UserFilter, UserResponse};
use crate::repository::UserRepository;
use crate::service::UserService;

/// Create the users router with all HTTP endpoints
pub fn router<R: UserRepository + 'static>(service: UserService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/verify-email", post(verify_email))
        .route("/{id}/change-password", post(change_password))
        .route("/login", post(login))
        .with_state(shared_service)
}

/// List response with pagination info
#[derive(Debug, Serialize)]
struct ListUsersResponse {
    data: Vec<UserResponse>,
    total: usize,
    limit: usize,
    offset: usize,
}

/// List users with optional filters
///
/// GET /users?email=test&role=admin&limit=10&offset=0
async fn list_users<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Query(filter): Query<UserFilter>,
) -> UserResult<Json<ListUsersResponse>> {
    let limit = filter.limit;
    let offset = filter.offset;
    let (users, total) = service.list_users(filter).await?;

    Ok(Json(ListUsersResponse {
        data: users,
        total,
        limit,
        offset,
    }))
}

/// Create a new user
///
/// POST /users
async fn create_user<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    ValidatedJson(input): ValidatedJson<CreateUser>,
) -> UserResult<impl IntoResponse> {
    let user = service.create_user(input).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Get a user by ID
///
/// GET /users/:id
async fn get_user<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Path(id): Path<Uuid>,
) -> UserResult<Json<UserResponse>> {
    let user = service.get_user(id).await?;
    Ok(Json(user))
}

/// Update a user
///
/// PUT /users/:id
async fn update_user<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Path(id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<UpdateUser>,
) -> UserResult<Json<UserResponse>> {
    let user = service.update_user(id, input).await?;
    Ok(Json(user))
}

/// Delete a user
///
/// DELETE /users/:id
async fn delete_user<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Path(id): Path<Uuid>,
) -> UserResult<impl IntoResponse> {
    service.delete_user(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Verify user email
///
/// POST /users/:id/verify-email
async fn verify_email<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Path(id): Path<Uuid>,
) -> UserResult<Json<UserResponse>> {
    let user = service.verify_email(id).await?;
    Ok(Json(user))
}

/// Change password request
#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

/// Change password response
#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

/// Change user password
///
/// POST /users/:id/change-password
async fn change_password<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    Path(id): Path<Uuid>,
    Json(input): Json<ChangePasswordRequest>,
) -> UserResult<Json<MessageResponse>> {
    service
        .change_password(id, &input.current_password, &input.new_password)
        .await?;

    Ok(Json(MessageResponse {
        message: "Password changed successfully".to_string(),
    }))
}

/// User login (verify credentials)
///
/// POST /users/login
async fn login<R: UserRepository>(
    State(service): State<Arc<UserService<R>>>,
    ValidatedJson(input): ValidatedJson<LoginRequest>,
) -> UserResult<Json<UserResponse>> {
    let user = service
        .verify_credentials(&input.email, &input.password)
        .await?;
    Ok(Json(user))
}

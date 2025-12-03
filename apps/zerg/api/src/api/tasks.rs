use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rpc::tasks::{CreateRequest, DeleteByIdRequest, GetByIdRequest, ListRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub completed: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskDto {
    pub title: String,
    pub description: String,
    pub completed: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List all tasks", body = Vec<TaskDto>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.tasks_client.write().await;

    match client
        .list(ListRequest {
            limit: String::new(),
            projection: vec![],
        })
        .await
    {
        Ok(response) => {
            let tasks: Vec<TaskDto> = response
                .into_inner()
                .data
                .into_iter()
                .map(|t| TaskDto {
                    id: t.id,
                    title: t.title,
                    description: t.description,
                    completed: t.completed,
                })
                .collect();
            (StatusCode::OK, Json(tasks)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.message().to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = TaskDto),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut client = state.tasks_client.write().await;

    match client
        .get_by_id(GetByIdRequest {
            id,
            projection: vec![],
        })
        .await
    {
        Ok(response) => {
            let task = response.into_inner();
            (
                StatusCode::OK,
                Json(TaskDto {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    completed: task.completed,
                }),
            )
                .into_response()
        }
        Err(e) => {
            let status = match e.code() {
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.message().to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/tasks",
    request_body = CreateTaskDto,
    responses(
        (status = 201, description = "Task created", body = TaskDto),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskDto>,
) -> impl IntoResponse {
    let mut client = state.tasks_client.write().await;

    match client
        .create(CreateRequest {
            title: payload.title,
            description: payload.description,
            completed: payload.completed.unwrap_or_else(|| "false".to_string()),
        })
        .await
    {
        Ok(response) => {
            let task = response.into_inner();
            (
                StatusCode::CREATED,
                Json(TaskDto {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    completed: task.completed,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.message().to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut client = state.tasks_client.write().await;

    match client.delete_by_id(DeleteByIdRequest { id }).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = match e.code() {
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.message().to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub fn router(state: crate::AppState) -> Router {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).delete(delete_task))
        .with_state(state)
}

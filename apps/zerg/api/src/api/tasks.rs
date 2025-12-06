use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use rpc::tasks::{CreateRequest, DeleteByIdRequest, GetByIdRequest, ListRequest, UpdateByIdRequest, Priority, Status};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// Conversion helpers
fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

fn bytes_to_uuid_string(bytes: Vec<u8>) -> Option<String> {
    Uuid::from_slice(&bytes).ok().map(|u| u.to_string())
}

fn opt_bytes_to_uuid_string(bytes: Option<Vec<u8>>) -> Option<String> {
    bytes.and_then(bytes_to_uuid_string)
}

fn string_to_priority_enum(s: &str) -> i32 {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low as i32,
        "high" => Priority::High as i32,
        "urgent" => Priority::Urgent as i32,
        _ => Priority::Medium as i32,
    }
}

fn priority_enum_to_string(p: i32) -> String {
    match Priority::try_from(p) {
        Ok(Priority::Low) => "low".to_string(),
        Ok(Priority::High) => "high".to_string(),
        Ok(Priority::Urgent) => "urgent".to_string(),
        _ => "medium".to_string(),
    }
}

fn string_to_status_enum(s: &str) -> i32 {
    match s.to_lowercase().replace('_', "").as_str() {
        "inprogress" => Status::InProgress as i32,
        "done" => Status::Done as i32,
        _ => Status::Todo as i32,
    }
}

fn status_enum_to_string(s: i32) -> String {
    match Status::try_from(s) {
        Ok(Status::InProgress) => "in_progress".to_string(),
        Ok(Status::Done) => "done".to_string(),
        _ => "todo".to_string(),
    }
}

fn timestamp_to_string(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .unwrap_or_else(|| Utc::now())
        .to_rfc3339()
}

fn opt_timestamp_to_string(ts: Option<i64>) -> Option<String> {
    ts.map(timestamp_to_string)
}

fn string_to_timestamp(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp())
}

fn opt_string_to_timestamp(s: Option<String>) -> Option<i64> {
    s.and_then(|s| string_to_timestamp(&s))
}

fn opt_uuid_string_to_bytes(s: Option<String>) -> Option<Vec<u8>> {
    s.and_then(|s| Uuid::parse_str(&s).ok().map(uuid_to_bytes))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub project_id: Option<String>,
    pub priority: String,
    pub status: String,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskDto {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub project_id: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_status")]
    pub status: String,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub project_id: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

fn default_priority() -> String {
    "medium".to_string()
}

fn default_status() -> String {
    "todo".to_string()
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
    let mut client = state.tasks_client.clone();

    match client
        .list(ListRequest {
            project_id: None,
            status: None,
            priority: None,
            completed: None,
            limit: 50,
            offset: 0,
        })
        .await
    {
        Ok(response) => {
            let tasks: Vec<TaskDto> = response
                .into_inner()
                .data
                .into_iter()
                .map(|t| TaskDto {
                    id: bytes_to_uuid_string(t.id).unwrap_or_default(),
                    title: t.title,
                    description: t.description,
                    completed: t.completed,
                    project_id: opt_bytes_to_uuid_string(t.project_id),
                    priority: priority_enum_to_string(t.priority),
                    status: status_enum_to_string(t.status),
                    due_date: opt_timestamp_to_string(t.due_date),
                    created_at: timestamp_to_string(t.created_at),
                    updated_at: timestamp_to_string(t.updated_at),
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
    let mut client = state.tasks_client.clone();

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid UUID format".to_string(),
                }),
            )
                .into_response()
        }
    };

    match client
        .get_by_id(GetByIdRequest {
            id: uuid_to_bytes(uuid),
        })
        .await
    {
        Ok(response) => {
            let task = response.into_inner();
            (
                StatusCode::OK,
                Json(TaskDto {
                    id: bytes_to_uuid_string(task.id).unwrap_or_default(),
                    title: task.title,
                    description: task.description,
                    completed: task.completed,
                    project_id: opt_bytes_to_uuid_string(task.project_id),
                    priority: priority_enum_to_string(task.priority),
                    status: status_enum_to_string(task.status),
                    due_date: opt_timestamp_to_string(task.due_date),
                    created_at: timestamp_to_string(task.created_at),
                    updated_at: timestamp_to_string(task.updated_at),
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
    let mut client = state.tasks_client.clone();

    match client
        .create(CreateRequest {
            title: payload.title,
            description: payload.description,
            project_id: opt_uuid_string_to_bytes(payload.project_id),
            priority: string_to_priority_enum(&payload.priority),
            status: string_to_status_enum(&payload.status),
            due_date: opt_string_to_timestamp(payload.due_date),
        })
        .await
    {
        Ok(response) => {
            let task = response.into_inner();
            (
                StatusCode::CREATED,
                Json(TaskDto {
                    id: bytes_to_uuid_string(task.id).unwrap_or_default(),
                    title: task.title,
                    description: task.description,
                    completed: task.completed,
                    project_id: opt_bytes_to_uuid_string(task.project_id),
                    priority: priority_enum_to_string(task.priority),
                    status: status_enum_to_string(task.status),
                    due_date: opt_timestamp_to_string(task.due_date),
                    created_at: timestamp_to_string(task.created_at),
                    updated_at: timestamp_to_string(task.updated_at),
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
    put,
    path = "/tasks/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTaskDto,
    responses(
        (status = 200, description = "Task updated", body = TaskDto),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskDto>,
) -> impl IntoResponse {
    let mut client = state.tasks_client.clone();

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid UUID format".to_string(),
                }),
            )
                .into_response()
        }
    };

    match client
        .update_by_id(UpdateByIdRequest {
            id: uuid_to_bytes(uuid),
            title: payload.title,
            description: payload.description,
            completed: payload.completed,
            project_id: opt_uuid_string_to_bytes(payload.project_id),
            priority: payload.priority.map(|p| string_to_priority_enum(&p)),
            status: payload.status.map(|s| string_to_status_enum(&s)),
            due_date: opt_string_to_timestamp(payload.due_date),
        })
        .await
    {
        Ok(response) => {
            let task = response.into_inner();
            (
                StatusCode::OK,
                Json(TaskDto {
                    id: bytes_to_uuid_string(task.id).unwrap_or_default(),
                    title: task.title,
                    description: task.description,
                    completed: task.completed,
                    project_id: opt_bytes_to_uuid_string(task.project_id),
                    priority: priority_enum_to_string(task.priority),
                    status: status_enum_to_string(task.status),
                    due_date: opt_timestamp_to_string(task.due_date),
                    created_at: timestamp_to_string(task.created_at),
                    updated_at: timestamp_to_string(task.updated_at),
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
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut client = state.tasks_client.clone();

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid UUID format".to_string(),
                }),
            )
                .into_response()
        }
    };

    match client
        .delete_by_id(DeleteByIdRequest {
            id: uuid_to_bytes(uuid),
        })
        .await
    {
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

pub fn router(state: crate::state::AppState) -> Router {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).put(update_task).delete(delete_task))
        .with_state(state)
}

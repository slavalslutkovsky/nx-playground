use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use domain_tasks::{CreateTask, PgTaskRepository, TaskFilter, TaskService, UpdateTask};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

// Helper to parse priority string to enum
fn parse_priority(s: &str) -> Result<domain_tasks::TaskPriority, String> {
    match s.to_lowercase().as_str() {
        "low" => Ok(domain_tasks::TaskPriority::Low),
        "medium" => Ok(domain_tasks::TaskPriority::Medium),
        "high" => Ok(domain_tasks::TaskPriority::High),
        "urgent" => Ok(domain_tasks::TaskPriority::Urgent),
        _ => Err(format!("Invalid priority: {}", s)),
    }
}

// Helper to parse status string to enum
fn parse_status(s: &str) -> Result<domain_tasks::TaskStatus, String> {
    match s.to_lowercase().replace('_', "").as_str() {
        "todo" => Ok(domain_tasks::TaskStatus::Todo),
        "inprogress" => Ok(domain_tasks::TaskStatus::InProgress),
        "done" => Ok(domain_tasks::TaskStatus::Done),
        _ => Err(format!("Invalid status: {}", s)),
    }
}

// Helper to convert priority enum to string
fn priority_to_string(priority: &domain_tasks::TaskPriority) -> String {
    match priority {
        domain_tasks::TaskPriority::Low => "low".to_string(),
        domain_tasks::TaskPriority::Medium => "medium".to_string(),
        domain_tasks::TaskPriority::High => "high".to_string(),
        domain_tasks::TaskPriority::Urgent => "urgent".to_string(),
    }
}

// Helper to convert status enum to string
fn status_to_string(status: &domain_tasks::TaskStatus) -> String {
    match status {
        domain_tasks::TaskStatus::Todo => "todo".to_string(),
        domain_tasks::TaskStatus::InProgress => "in_progress".to_string(),
        domain_tasks::TaskStatus::Done => "done".to_string(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks-direct",
    responses(
        (status = 200, description = "List all tasks (direct DB access)", body = Vec<TaskDto>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks-direct"
)]
pub async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);

    let filter = TaskFilter {
        project_id: None,
        status: None,
        priority: None,
        completed: None,
        limit: 50,
        offset: 0,
    };

    match service.list_tasks(filter).await {
        Ok(tasks) => {
            let tasks: Vec<TaskDto> = tasks
                .into_iter()
                .map(|t| TaskDto {
                    id: t.id.to_string(),
                    title: t.title,
                    description: t.description,
                    completed: t.completed,
                    project_id: t.project_id.map(|id| id.to_string()),
                    priority: priority_to_string(&t.priority),
                    status: status_to_string(&t.status),
                    due_date: t.due_date.map(|d| d.to_rfc3339()),
                    created_at: t.created_at.to_rfc3339(),
                    updated_at: t.updated_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(tasks)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks-direct/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found (direct DB access)", body = TaskDto),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks-direct"
)]
pub async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);

    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid task ID".to_string(),
                }),
            )
                .into_response()
        }
    };

    match service.get_task(task_id).await {
        Ok(task) => (
            StatusCode::OK,
            Json(TaskDto {
                id: task.id.to_string(),
                title: task.title,
                description: task.description,
                completed: task.completed,
                project_id: task.project_id.map(|id| id.to_string()),
                priority: priority_to_string(&task.priority),
                status: status_to_string(&task.status),
                due_date: task.due_date.map(|d| d.to_rfc3339()),
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/tasks-direct",
    request_body = CreateTaskDto,
    responses(
        (status = 201, description = "Task created (direct DB access)", body = TaskDto),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks-direct"
)]
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskDto>,
) -> impl IntoResponse {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);

    let priority = match parse_priority(&payload.priority) {
        Ok(p) => p,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let status = match parse_status(&payload.status) {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let project_id = if let Some(pid) = payload.project_id {
        match Uuid::parse_str(&pid) {
            Ok(uuid) => Some(uuid),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid project_id".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let due_date = if let Some(dd) = payload.due_date {
        match chrono::DateTime::parse_from_rfc3339(&dd) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid due_date format".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let input = CreateTask {
        title: payload.title,
        description: payload.description,
        project_id,
        priority,
        status,
        due_date,
    };

    match service.create_task(input).await {
        Ok(task) => (
            StatusCode::CREATED,
            Json(TaskDto {
                id: task.id.to_string(),
                title: task.title,
                description: task.description,
                completed: task.completed,
                project_id: task.project_id.map(|id| id.to_string()),
                priority: priority_to_string(&task.priority),
                status: status_to_string(&task.status),
                due_date: task.due_date.map(|d| d.to_rfc3339()),
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/tasks-direct/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTaskDto,
    responses(
        (status = 200, description = "Task updated (direct DB access)", body = TaskDto),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks-direct"
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskDto>,
) -> impl IntoResponse {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);

    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid task ID".to_string(),
                }),
            )
                .into_response()
        }
    };

    let priority = if let Some(p) = payload.priority {
        match parse_priority(&p) {
            Ok(priority) => Some(priority),
            Err(e) => {
                return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
            }
        }
    } else {
        None
    };

    let status = if let Some(s) = payload.status {
        match parse_status(&s) {
            Ok(status) => Some(status),
            Err(e) => {
                return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
            }
        }
    } else {
        None
    };

    let project_id = if let Some(pid) = payload.project_id {
        match Uuid::parse_str(&pid) {
            Ok(uuid) => Some(Some(uuid)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid project_id".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let due_date = if let Some(dd) = payload.due_date {
        match chrono::DateTime::parse_from_rfc3339(&dd) {
            Ok(dt) => Some(Some(dt.with_timezone(&chrono::Utc))),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid due_date format".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    let input = UpdateTask {
        title: payload.title,
        description: payload.description,
        completed: payload.completed,
        project_id,
        priority,
        status,
        due_date,
    };

    match service.update_task(task_id, input).await {
        Ok(task) => (
            StatusCode::OK,
            Json(TaskDto {
                id: task.id.to_string(),
                title: task.title,
                description: task.description,
                completed: task.completed,
                project_id: task.project_id.map(|id| id.to_string()),
                priority: priority_to_string(&task.priority),
                status: status_to_string(&task.status),
                due_date: task.due_date.map(|d| d.to_rfc3339()),
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/tasks-direct/{id}",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted (direct DB access)"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tasks-direct"
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);

    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid task ID".to_string(),
                }),
            )
                .into_response()
        }
    };

    match service.delete_task(task_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub fn router(state: crate::state::AppState) -> Router {
    Router::new()
        .route("/tasks-direct", get(list_tasks).post(create_task))
        .route(
            "/tasks-direct/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(state)
}

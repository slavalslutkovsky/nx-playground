//! Task routes with event publishing.
//!
//! Wraps the domain task handlers with NATS event publishing.

use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use domain_tasks::models::{CreateTask, Task, UpdateTask};
use rpc::tasks::tasks_service_client::TasksServiceClient;
use rpc::tasks::{CreateRequest, DeleteByIdRequest, GetByIdRequest, ListRequest, UpdateByIdRequest};
use tonic::transport::Channel;
use tracing::info;
use uuid::Uuid;

use crate::events::{EventPublisher, TaskCreatedEvent, TaskDeletedEvent, TaskUpdatedEvent};
use crate::state::AppState;

/// Combined state for task handlers
#[derive(Clone)]
pub struct TaskState {
    pub client: TasksServiceClient<Channel>,
    pub events: Option<EventPublisher>,
}

pub fn router(state: AppState) -> Router {
    let task_state = TaskState {
        client: state.tasks_client.clone(),
        events: state.events.clone(),
    };

    Router::new()
        .route("/", get(list_tasks).post(create_task))
        .route(
            "/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(task_state)
}

/// List tasks via gRPC
pub async fn list_tasks(
    State(mut state): State<TaskState>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let response = state
        .client
        .list(ListRequest {
            project_id: None,
            status: None,
            priority: None,
            completed: None,
            limit: 50,
            offset: 0,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let tasks = domain_tasks::conversions::list_response_to_tasks(response.into_inner())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tasks))
}

/// Get a task by ID via gRPC
pub async fn get_task(
    State(mut state): State<TaskState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid task ID".to_string()))?;

    let response = state
        .client
        .get_by_id(GetByIdRequest {
            id: domain_tasks::conversions::uuid_to_bytes(uuid),
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                (StatusCode::NOT_FOUND, "Task not found".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    let task: Task = response
        .into_inner()
        .try_into()
        .map_err(|e: String| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(task))
}

/// Create a new task via gRPC and publish event
pub async fn create_task(
    State(mut state): State<TaskState>,
    Json(input): Json<CreateTask>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response = state
        .client
        .create(CreateRequest::from(input))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task: Task = response
        .into_inner()
        .try_into()
        .map_err(|e: String| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Publish event if NATS is connected
    if let Some(ref events) = state.events {
        let event = TaskCreatedEvent::from(&task);
        info!(task_id = %task.id, "Publishing task.created event");
        events.task_created(&event).await;
    }

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task via gRPC and publish event
pub async fn update_task(
    State(mut state): State<TaskState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid task ID".to_string()))?;

    let mut request: UpdateByIdRequest = input.clone().into();
    request.id = domain_tasks::conversions::uuid_to_bytes(uuid);

    let response = state.client.update_by_id(request).await.map_err(|e| {
        if e.code() == tonic::Code::NotFound {
            (StatusCode::NOT_FOUND, "Task not found".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let task: Task = response
        .into_inner()
        .try_into()
        .map_err(|e: String| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Publish event if NATS is connected
    if let Some(ref events) = state.events {
        let event = TaskUpdatedEvent {
            id: task.id.to_string(),
            title: input.title,
            description: input.description,
            status: input.status.map(|s| format!("{:?}", s)),
            priority: input.priority.map(|p| format!("{:?}", p)),
        };
        info!(task_id = %task.id, "Publishing task.updated event");
        events.task_updated(&event).await;
    }

    Ok(Json(task))
}

/// Delete a task via gRPC and publish event
pub async fn delete_task(
    State(mut state): State<TaskState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid task ID".to_string()))?;

    state
        .client
        .delete_by_id(DeleteByIdRequest {
            id: domain_tasks::conversions::uuid_to_bytes(uuid),
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                (StatusCode::NOT_FOUND, "Task not found".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    // Publish event if NATS is connected
    if let Some(ref events) = state.events {
        let event = TaskDeletedEvent { id: id.clone() };
        info!(task_id = %id, "Publishing task.deleted event");
        events.task_deleted(&event).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

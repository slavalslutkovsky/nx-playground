use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rpc::tasks::{CreateRequest, GetByIdRequest, UpdateByIdRequest, DeleteByIdRequest, ListRequest};
use rpc::tasks::tasks_service_client::TasksServiceClient;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::error::{TaskError, TaskResult};
use crate::models::{CreateTask, Task, UpdateTask};

// Import proto conversion helpers
use super::proto_conversions::*;

/// List tasks via gRPC
#[utoipa::path(
    get,
    path = "",
    tag = "tasks",
    responses(
        (status = 200, description = "List of tasks via gRPC", body = Vec<Task>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_tasks(
    State(mut client): State<TasksServiceClient<Channel>>,
) -> TaskResult<Json<Vec<Task>>> {
    let response = client.list(ListRequest {
        project_id: None,
        status: None,
        priority: None,
        completed: None,
        limit: 50,
        offset: 0,
    })
    .await
    .map_err(|e| TaskError::Internal(e.to_string()))?;

    let tasks: Vec<Task> = response
        .into_inner()
        .data
        .into_iter()
        .map(|proto_task| Task {
            id: bytes_to_uuid(&proto_task.id).unwrap_or_else(|_| Uuid::new_v4()),
            title: proto_task.title,
            description: proto_task.description,
            completed: proto_task.completed,
            project_id: opt_bytes_to_uuid(proto_task.project_id).ok().flatten(),
            priority: proto_priority_to_domain(proto_task.priority).unwrap_or_default(),
            status: proto_status_to_domain(proto_task.status).unwrap_or_default(),
            due_date: opt_timestamp_to_datetime(proto_task.due_date),
            created_at: timestamp_to_datetime(proto_task.created_at),
            updated_at: timestamp_to_datetime(proto_task.updated_at),
        })
        .collect();

    Ok(Json(tasks))
}

/// Get a task by ID via gRPC
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found via gRPC", body = Task),
        (status = 400, description = "Invalid task ID"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_task(
    State(mut client): State<TasksServiceClient<Channel>>,
    Path(id): Path<String>,
) -> TaskResult<impl IntoResponse> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    let response = client.get_by_id(GetByIdRequest {
        id: uuid_to_bytes(uuid),
    })
    .await
    .map_err(|e| {
        if e.code() == tonic::Code::NotFound {
            TaskError::NotFound(uuid)
        } else {
            TaskError::Internal(e.to_string())
        }
    })?;

    let proto_task = response.into_inner();
    let task = Task {
        id: bytes_to_uuid(&proto_task.id).unwrap_or(uuid),
        title: proto_task.title,
        description: proto_task.description,
        completed: proto_task.completed,
        project_id: opt_bytes_to_uuid(proto_task.project_id).ok().flatten(),
        priority: proto_priority_to_domain(proto_task.priority).unwrap_or_default(),
        status: proto_status_to_domain(proto_task.status).unwrap_or_default(),
        due_date: opt_timestamp_to_datetime(proto_task.due_date),
        created_at: timestamp_to_datetime(proto_task.created_at),
        updated_at: timestamp_to_datetime(proto_task.updated_at),
    };

    Ok(Json(task))
}

/// Create a new task via gRPC
#[utoipa::path(
    post,
    path = "",
    tag = "tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task created successfully via gRPC", body = Task),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_task(
    State(mut client): State<TasksServiceClient<Channel>>,
    Json(input): Json<CreateTask>,
) -> TaskResult<impl IntoResponse> {
    let request = CreateRequest {
        title: input.title,
        description: input.description,
        project_id: opt_uuid_to_bytes(input.project_id),
        priority: domain_priority_to_proto(&input.priority),
        status: domain_status_to_proto(&input.status),
        due_date: opt_datetime_to_timestamp(input.due_date),
    };

    let response = client.create(request)
        .await
        .map_err(|e| TaskError::Internal(e.to_string()))?;

    let proto_task = response.into_inner();
    let task = Task {
        id: bytes_to_uuid(&proto_task.id).unwrap_or_else(|_| Uuid::new_v4()),
        title: proto_task.title,
        description: proto_task.description,
        completed: proto_task.completed,
        project_id: opt_bytes_to_uuid(proto_task.project_id).ok().flatten(),
        priority: proto_priority_to_domain(proto_task.priority).unwrap_or_default(),
        status: proto_status_to_domain(proto_task.status).unwrap_or_default(),
        due_date: opt_timestamp_to_datetime(proto_task.due_date),
        created_at: timestamp_to_datetime(proto_task.created_at),
        updated_at: timestamp_to_datetime(proto_task.updated_at),
    };

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task via gRPC
#[utoipa::path(
    put,
    path = "/{id}",
    tag = "tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTask,
    responses(
        (status = 200, description = "Task updated successfully via gRPC", body = Task),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_task(
    State(mut client): State<TasksServiceClient<Channel>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTask>,
) -> TaskResult<impl IntoResponse> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    let request = UpdateByIdRequest {
        id: uuid_to_bytes(uuid),
        title: input.title,
        description: input.description,
        completed: input.completed,
        project_id: input.project_id.and_then(opt_uuid_to_bytes),
        priority: input.priority.map(|p| domain_priority_to_proto(&p)),
        status: input.status.map(|s| domain_status_to_proto(&s)),
        due_date: input.due_date.and_then(opt_datetime_to_timestamp),
    };

    let response = client.update_by_id(request)
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                TaskError::NotFound(uuid)
            } else {
                TaskError::Internal(e.to_string())
            }
        })?;

    let proto_task = response.into_inner();
    let task = Task {
        id: bytes_to_uuid(&proto_task.id).unwrap_or(uuid),
        title: proto_task.title,
        description: proto_task.description,
        completed: proto_task.completed,
        project_id: opt_bytes_to_uuid(proto_task.project_id).ok().flatten(),
        priority: proto_priority_to_domain(proto_task.priority).unwrap_or_default(),
        status: proto_status_to_domain(proto_task.status).unwrap_or_default(),
        due_date: opt_timestamp_to_datetime(proto_task.due_date),
        created_at: timestamp_to_datetime(proto_task.created_at),
        updated_at: timestamp_to_datetime(proto_task.updated_at),
    };

    Ok(Json(task))
}

/// Delete a task via gRPC
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted successfully via gRPC"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_task(
    State(mut client): State<TasksServiceClient<Channel>>,
    Path(id): Path<String>,
) -> TaskResult<impl IntoResponse> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    client.delete_by_id(DeleteByIdRequest {
        id: uuid_to_bytes(uuid),
    })
    .await
    .map_err(|e| {
        if e.code() == tonic::Code::NotFound {
            TaskError::NotFound(uuid)
        } else {
            TaskError::Internal(e.to_string())
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

mod direct;
mod grpc;
pub mod stream;

use axum::{routing::get, Router};
use rpc::tasks::tasks_service_client::TasksServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;
use utoipa::OpenApi;

use crate::models::{CreateTask, Task, UpdateTask};
use crate::repository::TaskRepository;
use crate::service::TaskService;

// Re-export stream types for external use
pub use stream::StreamState;

/// OpenAPI documentation for Tasks API (Direct DB)
#[derive(OpenApi)]
#[openapi(
    paths(
        direct::list_tasks,
        direct::get_task,
        direct::create_task,
        direct::update_task,
        direct::delete_task,
    ),
    components(
        schemas(Task, CreateTask, UpdateTask)
    ),
    tags(
        (name = "tasks-direct", description = "Direct database task operations")
    )
)]
pub struct DirectApiDoc;

/// OpenAPI documentation for Tasks API (gRPC)
#[derive(OpenApi)]
#[openapi(
    paths(
        grpc::list_tasks,
        grpc::get_task,
        grpc::create_task,
        grpc::update_task,
        grpc::delete_task,
    ),
    components(
        schemas(Task, CreateTask, UpdateTask)
    ),
    tags(
        (name = "tasks", description = "gRPC-backed task operations")
    )
)]
pub struct GrpcApiDoc;

/// Create router for direct DB-backed handlers
pub fn direct_router<R: TaskRepository + 'static>(service: TaskService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/", get(direct::list_tasks).post(direct::create_task))
        .route("/{id}", get(direct::get_task).put(direct::update_task).delete(direct::delete_task))
        .with_state(shared_service)
}

/// Create router for gRPC-backed handlers
pub fn grpc_router(client: TasksServiceClient<Channel>) -> Router {
    Router::new()
        .route("/", get(grpc::list_tasks).post(grpc::create_task))
        .route("/{id}", get(grpc::get_task).put(grpc::update_task).delete(grpc::delete_task))
        .with_state(client)
}

/// Create router for stream-backed handlers (fire-and-forget, returns 202 Accepted)
///
/// These handlers queue commands to Redis streams and return immediately.
/// Use this for measuring raw queueing throughput.
pub fn stream_async_router(state: StreamState) -> Router {
    Router::new()
        .route("/", get(stream::list_tasks_async).post(stream::create_task_async))
        .route(
            "/{id}",
            get(stream::get_task_async)
                .put(stream::update_task_async)
                .delete(stream::delete_task_async),
        )
        .with_state(state)
}

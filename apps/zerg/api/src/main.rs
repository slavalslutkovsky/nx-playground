use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use domain_projects::{handlers as projects_handlers, PgProjectRepository, ProjectService};
use domain_users::{handlers as users_handlers, InMemoryUserRepository, UserService};
use rpc::tasks::{
    tasks_service_client::TasksServiceClient, CreateRequest, DeleteByIdRequest, GetByIdRequest,
    ListRequest,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone)]
struct AppState {
    tasks_client: Arc<RwLock<TasksServiceClient<Channel>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskDto {
    id: String,
    title: String,
    description: String,
    completed: String,
}

#[derive(Debug, Deserialize)]
struct CreateTaskDto {
    title: String,
    description: String,
    completed: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
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

async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
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

async fn create_task(
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

async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let tasks_addr =
        std::env::var("TASKS_SERVICE_ADDR").unwrap_or_else(|_| "http://[::1]:50051".to_string());

    info!("Connecting to TasksService at {}", tasks_addr);

    let tasks_client = TasksServiceClient::connect(tasks_addr).await?;

    let state = AppState {
        tasks_client: Arc::new(RwLock::new(tasks_client)),
    };

    // Initialize database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://myuser:mypassword@localhost/mydatabase".to_string());

    info!("Connecting to database");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("Database connection established");

    // Initialize projects domain with PostgreSQL
    let projects_repo = PgProjectRepository::new(pool.clone());
    let projects_service = ProjectService::new(projects_repo);

    // Initialize users' domain
    let users_repo = InMemoryUserRepository::new();
    let users_service = UserService::new(users_repo);

    let app = Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).delete(delete_task))
        .with_state(state)
        .nest("/projects", projects_handlers::router(projects_service))
        .nest("/users", users_handlers::router(users_service));

    let addr = "0.0.0.0:3000";
    info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

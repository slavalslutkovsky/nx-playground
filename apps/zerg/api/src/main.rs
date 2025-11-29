use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use domain_cloud_resources::{
    handlers as cloud_resources_handlers, CloudResourceService, PgCloudResourceRepository,
};
use domain_projects::{handlers as projects_handlers, PgProjectRepository, ProjectService};
use domain_users::{handlers as users_handlers, InMemoryUserRepository, UserService};
use migration::{Migrator, MigratorTrait};
use rpc::tasks::{
    tasks_service_client::TasksServiceClient, CreateRequest, DeleteByIdRequest, GetByIdRequest,
    ListRequest,
};
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::info;

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

use core_config::tracing::init_tracing;
use zerg_api::config::Config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing with environment-aware configuration
    init_tracing(&config.environment);

    let tasks_addr =
        std::env::var("TASKS_SERVICE_ADDR").unwrap_or_else(|_| "http://[::1]:50051".to_string());

    info!("Connecting to TasksService at {}", tasks_addr);

    let tasks_client = TasksServiceClient::connect(tasks_addr).await?;

    let state = AppState {
        tasks_client: Arc::new(RwLock::new(tasks_client)),
    };

    // Initialize database connection with Sea-ORM
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://myuser:mypassword@localhost/mydatabase".to_string());

    info!("Connecting to database");

    let db = Database::connect(&database_url).await?;

    info!("Database connection established");

    // Conditional migrations based on environment
    // Set RUN_MIGRATIONS=true for development, or use the separate migrate binary for production
    if std::env::var("RUN_MIGRATIONS").is_ok() {
        info!("Running database migrations (RUN_MIGRATIONS is set)");
        Migrator::up(&db, None).await?;
        info!("Migrations completed successfully");
    } else {
        info!("Skipping automatic migrations. Use 'cargo run --bin migrate' to run migrations separately");
    }

    // Initialize projects domain with PostgreSQL
    let projects_repo = PgProjectRepository::new(db.clone());
    let projects_service = ProjectService::new(projects_repo);

    // Initialize cloud resources domain with PostgreSQL
    let cloud_resources_repo = PgCloudResourceRepository::new(db.clone());
    let cloud_resources_service = CloudResourceService::new(cloud_resources_repo);

    // Initialize users' domain (still in-memory)
    let users_repo = InMemoryUserRepository::new();
    let users_service = UserService::new(users_repo);

    let app = Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).delete(delete_task))
        .with_state(state)
        .nest("/projects", projects_handlers::router(projects_service))
        .nest(
            "/cloud-resources",
            cloud_resources_handlers::router(cloud_resources_service),
        )
        .nest("/users", users_handlers::router(users_service));

    let addr = "0.0.0.0:3000";
    info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ProjectResult;
use crate::models::{CreateProject, Project, ProjectFilter, UpdateProject};
use crate::repository::ProjectRepository;
use crate::service::ProjectService;

/// Create the project router with all HTTP endpoints
pub fn router<R: ProjectRepository + 'static>(service: ProjectService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route(
            "/{id}",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route("/{id}/activate", post(activate_project))
        .route("/{id}/suspend", post(suspend_project))
        .route("/{id}/archive", post(archive_project))
        .with_state(shared_service)
}

/// List projects with optional filters
///
/// GET /projects?user_id=xxx&cloud_provider=aws&limit=10
async fn list_projects<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Query(filter): Query<ProjectFilter>,
) -> ProjectResult<Json<Vec<Project>>> {
    let projects = service.list_projects(filter).await?;
    Ok(Json(projects))
}

/// Create a new project
///
/// POST /projects
async fn create_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Json(input): Json<CreateProject>,
) -> ProjectResult<impl IntoResponse> {
    let project = service.create_project(input).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

/// Get a project by ID
///
/// GET /projects/:id
async fn get_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
) -> ProjectResult<Json<Project>> {
    let project = service.get_project(id).await?;
    Ok(Json(project))
}

/// Update a project
///
/// PUT /projects/:id
async fn update_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProject>,
) -> ProjectResult<Json<Project>> {
    let project = service.update_project(id, input).await?;
    Ok(Json(project))
}

/// Delete a project
///
/// DELETE /projects/:id
async fn delete_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
) -> ProjectResult<impl IntoResponse> {
    service.delete_project(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Activate a project
///
/// POST /projects/:id/activate
async fn activate_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
) -> ProjectResult<Json<Project>> {
    let project = service.activate_project(id).await?;
    Ok(Json(project))
}

/// Suspend a project
///
/// POST /projects/:id/suspend
async fn suspend_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
) -> ProjectResult<Json<Project>> {
    let project = service.suspend_project(id).await?;
    Ok(Json(project))
}

/// Archive a project
///
/// POST /projects/:id/archive
async fn archive_project<R: ProjectRepository>(
    State(service): State<Arc<ProjectService<R>>>,
    Path(id): Path<Uuid>,
) -> ProjectResult<Json<Project>> {
    let project = service.archive_project(id).await?;
    Ok(Json(project))
}

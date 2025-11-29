use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::CloudResourceResult,
    models::{CloudResourceFilter, CreateCloudResource, UpdateCloudResource},
    repository::CloudResourceRepository,
    service::CloudResourceService,
};

/// Create Axum router for cloud resource endpoints
pub fn router<R>(service: CloudResourceService<R>) -> Router
where
    R: CloudResourceRepository + 'static,
{
    let service = Arc::new(service);

    Router::new()
        .route("/", post(create_cloud_resource).get(list_cloud_resources))
        .route(
            "/{id}",
            get(get_cloud_resource)
                .put(update_cloud_resource)
                .delete(delete_cloud_resource),
        )
        .route("/project/{project_id}", get(list_by_project))
        .route("/{id}/soft-delete", post(soft_delete_cloud_resource))
        .with_state(service)
}

/// POST /cloud-resources - Create a new cloud resource
async fn create_cloud_resource<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Json(input): Json<CreateCloudResource>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    let resource = service.create(input).await?;
    Ok((StatusCode::CREATED, Json(resource)))
}

/// GET /cloud-resources/:id - Get cloud resource by ID
async fn get_cloud_resource<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Path(id): Path<Uuid>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    let resource = service.get(id).await?;
    Ok(Json(resource))
}

/// GET /cloud-resources - List cloud resources with filters
async fn list_cloud_resources<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Query(filter): Query<CloudResourceFilter>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    let resources = service.list(filter).await?;
    Ok(Json(resources))
}

/// GET /cloud-resources/project/:project_id - List cloud resources by project
async fn list_by_project<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Path(project_id): Path<Uuid>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    let resources = service.list_by_project(project_id).await?;
    Ok(Json(resources))
}

/// PUT /cloud-resources/:id - Update cloud resource
async fn update_cloud_resource<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCloudResource>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    let resource = service.update(id, input).await?;
    Ok(Json(resource))
}

/// DELETE /cloud-resources/:id - Delete cloud resource (hard delete)
async fn delete_cloud_resource<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Path(id): Path<Uuid>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    service.delete(id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Cloud resource deleted successfully" })),
    ))
}

/// POST /cloud-resources/:id/soft-delete - Soft delete cloud resource
async fn soft_delete_cloud_resource<R>(
    State(service): State<Arc<CloudResourceService<R>>>,
    Path(id): Path<Uuid>,
) -> CloudResourceResult<impl IntoResponse>
where
    R: CloudResourceRepository,
{
    service.soft_delete(id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Cloud resource soft deleted successfully" })),
    ))
}

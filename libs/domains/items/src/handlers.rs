use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_helpers::{
    UuidPath, ValidatedJson,
    errors::responses::{
        BadRequestUuidResponse, BadRequestValidationResponse, ConflictResponse,
        InternalServerErrorResponse, NotFoundResponse,
    },
};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::error::ItemResult;
use crate::models::{CreateItem, Item, ItemFilter, UpdateItem};
use crate::repository::ItemRepository;
use crate::service::ItemService;

/// OpenAPI documentation for Items API
#[derive(OpenApi)]
#[openapi(
    paths(
        list_items,
        create_item,
        get_item,
        update_item,
        delete_item,
        activate_item,
        deactivate_item,
        archive_item,
        count_items,
    ),
    components(
        schemas(Item, CreateItem, UpdateItem, ItemFilter),
        responses(
            NotFoundResponse,
            BadRequestValidationResponse,
            BadRequestUuidResponse,
            ConflictResponse,
            InternalServerErrorResponse
        )
    ),
    tags(
        (name = "Items", description = "Item management endpoints (MongoDB)")
    )
)]
pub struct ApiDoc;

/// Create the items router with all HTTP endpoints
pub fn router<R: ItemRepository + 'static>(service: ItemService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/count", get(count_items))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
        .route("/{id}/activate", post(activate_item))
        .route("/{id}/deactivate", post(deactivate_item))
        .route("/{id}/archive", post(archive_item))
        .with_state(shared_service)
}

/// List items with optional filters
#[utoipa::path(
    get,
    path = "",
    tag = "Items",
    params(ItemFilter),
    responses(
        (status = 200, description = "List of items", body = Vec<Item>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_items<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    Query(filter): Query<ItemFilter>,
) -> ItemResult<Json<Vec<Item>>> {
    let items = service.list_items(filter).await?;
    Ok(Json(items))
}

/// Create a new item
#[utoipa::path(
    post,
    path = "",
    tag = "Items",
    request_body = CreateItem,
    responses(
        (status = 201, description = "Item created successfully", body = Item),
        (status = 400, response = BadRequestValidationResponse),
        (status = 409, response = ConflictResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn create_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    ValidatedJson(input): ValidatedJson<CreateItem>,
) -> ItemResult<impl IntoResponse> {
    let item = service.create_item(input).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Get an item by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
) -> ItemResult<Json<Item>> {
    let item = service.get_item(id).await?;
    Ok(Json(item))
}

/// Update an item
#[utoipa::path(
    put,
    path = "/{id}",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    request_body = UpdateItem,
    responses(
        (status = 200, description = "Item updated successfully", body = Item),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn update_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(input): ValidatedJson<UpdateItem>,
) -> ItemResult<Json<Item>> {
    let item = service.update_item(id, input).await?;
    Ok(Json(item))
}

/// Delete an item
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn delete_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
) -> ItemResult<impl IntoResponse> {
    service.delete_item(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Activate an item
#[utoipa::path(
    post,
    path = "/{id}/activate",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item activated successfully", body = Item),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn activate_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
) -> ItemResult<Json<Item>> {
    let item = service.activate_item(id).await?;
    Ok(Json(item))
}

/// Deactivate an item
#[utoipa::path(
    post,
    path = "/{id}/deactivate",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item deactivated successfully", body = Item),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn deactivate_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
) -> ItemResult<Json<Item>> {
    let item = service.deactivate_item(id).await?;
    Ok(Json(item))
}

/// Archive an item
#[utoipa::path(
    post,
    path = "/{id}/archive",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item archived successfully", body = Item),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn archive_item<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    UuidPath(id): UuidPath,
) -> ItemResult<Json<Item>> {
    let item = service.archive_item(id).await?;
    Ok(Json(item))
}

/// Count items matching a filter
#[utoipa::path(
    get,
    path = "/count",
    tag = "Items",
    params(ItemFilter),
    responses(
        (status = 200, description = "Item count", body = u64),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn count_items<R: ItemRepository>(
    State(service): State<Arc<ItemService<R>>>,
    Query(filter): Query<ItemFilter>,
) -> ItemResult<Json<u64>> {
    let count = service.count_items(filter).await?;
    Ok(Json(count))
}

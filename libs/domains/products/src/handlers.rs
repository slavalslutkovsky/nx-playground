//! HTTP handlers for Products API

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_helpers::{
    errors::responses::{
        BadRequestUuidResponse, BadRequestValidationResponse, ConflictResponse,
        InternalServerErrorResponse, NotFoundResponse,
    },
    UuidPath, ValidatedJson,
};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::error::ProductResult;
use crate::models::{
    CreateProduct, Product, ProductCategory, ProductFilter, ProductStatus, ReservationResult,
    StockAdjustment, StockReservation, UpdateProduct,
};
use crate::repository::ProductRepository;
use crate::service::ProductService;

/// OpenAPI documentation for Products API
#[derive(OpenApi)]
#[openapi(
    paths(
        list_products,
        create_product,
        get_product,
        update_product,
        delete_product,
        search_products,
        get_by_sku,
        get_by_barcode,
        get_by_category,
        count_products,
        adjust_stock,
        reserve_stock,
        release_stock,
        commit_stock,
        get_low_stock,
        activate_product,
        deactivate_product,
        discontinue_product,
    ),
    components(
        schemas(
            Product, CreateProduct, UpdateProduct, ProductFilter,
            ProductStatus, ProductCategory, StockAdjustment,
            StockReservation, ReservationResult
        ),
        responses(
            NotFoundResponse,
            BadRequestValidationResponse,
            BadRequestUuidResponse,
            ConflictResponse,
            InternalServerErrorResponse
        )
    ),
    tags(
        (name = "Products", description = "Product management endpoints")
    )
)]
pub struct ApiDoc;

/// Create the products router with all HTTP endpoints
pub fn router<R: ProductRepository + 'static>(service: ProductService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/", get(list_products).post(create_product))
        .route("/count", get(count_products))
        .route("/search", get(search_products))
        .route("/low-stock", get(get_low_stock))
        .route("/sku/{sku}", get(get_by_sku))
        .route("/barcode/{barcode}", get(get_by_barcode))
        .route("/category/{category}", get(get_by_category))
        .route(
            "/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/{id}/stock", post(adjust_stock))
        .route("/{id}/reserve", post(reserve_stock))
        .route("/{id}/release", post(release_stock))
        .route("/{id}/commit", post(commit_stock))
        .route("/{id}/activate", post(activate_product))
        .route("/{id}/deactivate", post(deactivate_product))
        .route("/{id}/discontinue", post(discontinue_product))
        .with_state(shared_service)
}

/// List products with optional filters
#[utoipa::path(
    get,
    path = "",
    tag = "Products",
    params(ProductFilter),
    responses(
        (status = 200, description = "List of products", body = Vec<Product>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_products<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    Query(filter): Query<ProductFilter>,
) -> ProductResult<Json<Vec<Product>>> {
    let products = service.list_products(filter).await?;
    Ok(Json(products))
}

/// Create a new product
#[utoipa::path(
    post,
    path = "",
    tag = "Products",
    request_body = CreateProduct,
    responses(
        (status = 201, description = "Product created successfully", body = Product),
        (status = 400, response = BadRequestValidationResponse),
        (status = 409, response = ConflictResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn create_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    ValidatedJson(input): ValidatedJson<CreateProduct>,
) -> ProductResult<impl IntoResponse> {
    let product = service.create_product(input).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

/// Get a product by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product found", body = Product),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
) -> ProductResult<Json<Product>> {
    let product = service.get_product(id).await?;
    Ok(Json(product))
}

/// Update a product
#[utoipa::path(
    put,
    path = "/{id}",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = UpdateProduct,
    responses(
        (status = 200, description = "Product updated successfully", body = Product),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn update_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(input): ValidatedJson<UpdateProduct>,
) -> ProductResult<Json<Product>> {
    let product = service.update_product(id, input).await?;
    Ok(Json(product))
}

/// Delete a product
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 204, description = "Product deleted successfully"),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn delete_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
) -> ProductResult<impl IntoResponse> {
    service.delete_product(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Search query parameters
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip
    #[serde(default)]
    pub offset: u64,
}

fn default_limit() -> i64 {
    50
}

/// Search products by text query
#[utoipa::path(
    get,
    path = "/search",
    tag = "Products",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = Vec<Product>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn search_products<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    Query(query): Query<SearchQuery>,
) -> ProductResult<Json<Vec<Product>>> {
    let products = service
        .search_products(&query.q, query.limit, query.offset)
        .await?;
    Ok(Json(products))
}

/// Get a product by SKU
#[utoipa::path(
    get,
    path = "/sku/{sku}",
    tag = "Products",
    params(
        ("sku" = String, Path, description = "Product SKU")
    ),
    responses(
        (status = 200, description = "Product found", body = Product),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_by_sku<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    axum::extract::Path(sku): axum::extract::Path<String>,
) -> ProductResult<Json<Product>> {
    let product = service.get_by_sku(&sku).await?;
    Ok(Json(product))
}

/// Get a product by barcode
#[utoipa::path(
    get,
    path = "/barcode/{barcode}",
    tag = "Products",
    params(
        ("barcode" = String, Path, description = "Product barcode (UPC, EAN, etc.)")
    ),
    responses(
        (status = 200, description = "Product found", body = Product),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_by_barcode<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    axum::extract::Path(barcode): axum::extract::Path<String>,
) -> ProductResult<Json<Product>> {
    let product = service.get_by_barcode(&barcode).await?;
    Ok(Json(product))
}

/// Category query parameters
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct CategoryQuery {
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip
    #[serde(default)]
    pub offset: u64,
}

/// Get products by category
#[utoipa::path(
    get,
    path = "/category/{category}",
    tag = "Products",
    params(
        ("category" = ProductCategory, Path, description = "Product category"),
        CategoryQuery
    ),
    responses(
        (status = 200, description = "Products in category", body = Vec<Product>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_by_category<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    axum::extract::Path(category): axum::extract::Path<ProductCategory>,
    Query(query): Query<CategoryQuery>,
) -> ProductResult<Json<Vec<Product>>> {
    let products = service
        .get_by_category(category, query.limit, query.offset)
        .await?;
    Ok(Json(products))
}

/// Count products matching a filter
#[utoipa::path(
    get,
    path = "/count",
    tag = "Products",
    params(ProductFilter),
    responses(
        (status = 200, description = "Product count", body = u64),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn count_products<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    Query(filter): Query<ProductFilter>,
) -> ProductResult<Json<u64>> {
    let count = service.count_products(filter).await?;
    Ok(Json(count))
}

/// Adjust product stock
#[utoipa::path(
    post,
    path = "/{id}/stock",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = StockAdjustment,
    responses(
        (status = 200, description = "Stock adjusted successfully", body = Product),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn adjust_stock<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(adjustment): ValidatedJson<StockAdjustment>,
) -> ProductResult<Json<Product>> {
    let product = service.adjust_stock(id, adjustment).await?;
    Ok(Json(product))
}

/// Reserve stock for an order
#[utoipa::path(
    post,
    path = "/{id}/reserve",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = StockReservation,
    responses(
        (status = 200, description = "Stock reserved", body = ReservationResult),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn reserve_stock<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(reservation): ValidatedJson<StockReservation>,
) -> ProductResult<Json<ReservationResult>> {
    let result = service.reserve_stock(id, reservation).await?;
    Ok(Json(result))
}

/// Release quantity request
#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ReleaseQuantity {
    /// Quantity to release
    #[validate(range(min = 1))]
    pub quantity: i32,
}

/// Release reserved stock
#[utoipa::path(
    post,
    path = "/{id}/release",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = ReleaseQuantity,
    responses(
        (status = 200, description = "Stock released", body = Product),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn release_stock<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(input): ValidatedJson<ReleaseQuantity>,
) -> ProductResult<Json<Product>> {
    let product = service.release_stock(id, input.quantity).await?;
    Ok(Json(product))
}

/// Commit quantity request
#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct CommitQuantity {
    /// Quantity to commit
    #[validate(range(min = 1))]
    pub quantity: i32,
}

/// Commit reserved stock (after order completion)
#[utoipa::path(
    post,
    path = "/{id}/commit",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = CommitQuantity,
    responses(
        (status = 200, description = "Stock committed", body = Product),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn commit_stock<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(input): ValidatedJson<CommitQuantity>,
) -> ProductResult<Json<Product>> {
    let product = service.commit_stock(id, input.quantity).await?;
    Ok(Json(product))
}

/// Low stock query parameters
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct LowStockQuery {
    /// Stock threshold
    #[serde(default = "default_threshold")]
    pub threshold: i32,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_threshold() -> i32 {
    10
}

/// Get products with low stock
#[utoipa::path(
    get,
    path = "/low-stock",
    tag = "Products",
    params(LowStockQuery),
    responses(
        (status = 200, description = "Low stock products", body = Vec<Product>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_low_stock<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    Query(query): Query<LowStockQuery>,
) -> ProductResult<Json<Vec<Product>>> {
    let products = service.get_low_stock(query.threshold, query.limit).await?;
    Ok(Json(products))
}

/// Activate a product
#[utoipa::path(
    post,
    path = "/{id}/activate",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product activated", body = Product),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn activate_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
) -> ProductResult<Json<Product>> {
    let product = service.activate_product(id).await?;
    Ok(Json(product))
}

/// Deactivate a product
#[utoipa::path(
    post,
    path = "/{id}/deactivate",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product deactivated", body = Product),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn deactivate_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
) -> ProductResult<Json<Product>> {
    let product = service.deactivate_product(id).await?;
    Ok(Json(product))
}

/// Discontinue a product
#[utoipa::path(
    post,
    path = "/{id}/discontinue",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product discontinued", body = Product),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn discontinue_product<R: ProductRepository>(
    State(service): State<Arc<ProductService<R>>>,
    UuidPath(id): UuidPath,
) -> ProductResult<Json<Product>> {
    let product = service.discontinue_product(id).await?;
    Ok(Json(product))
}

//! HTTP handlers for pricing domain

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_helpers::{
    errors::responses::{
        BadRequestUuidResponse, BadRequestValidationResponse, InternalServerErrorResponse,
        NotFoundResponse,
    },
    UuidPath, ValidatedJson,
};
use core_proc_macros::ApiResource;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::cncf_client;
use crate::cncf_models::{
    get_cncf_tools, CncfMaturity, CncfTool, CncfToolCategory, CostRecommendation, DeploymentMode,
    InfrastructureCostComparison, ManagedServiceEquivalent, OpsHoursEstimate,
    ResourceRequirements, TcoCalculationRequest, TcoCalculationResult,
};
use crate::entity;
use crate::error::PricingResult;
use crate::models::{
    CloudProvider, CreatePriceEntry, Currency, Money, PriceComparison, PriceEntry, PriceFilter,
    PricingUnit, ResourceType, UpdatePriceEntry,
};
use crate::repository::PricingRepository;
use crate::service::PricingService;
use crate::tco_calculator::TcoCalculator;

/// OpenAPI documentation for Pricing API
#[derive(OpenApi)]
#[openapi(
    paths(
        list_prices,
        create_price,
        get_price,
        update_price,
        delete_price,
        compare_prices,
        get_price_stats,
        list_cncf_tools,
        get_cncf_tool,
        list_cncf_landscape,
        get_cncf_category,
        get_cncf_recommendations,
        calculate_tco,
        compare_infrastructure,
    ),
    components(
        schemas(
            PriceEntry,
            CreatePriceEntry,
            UpdatePriceEntry,
            PriceFilter,
            PriceComparison,
            CompareQuery,
            PriceStats,
            CloudProvider,
            ResourceType,
            PricingUnit,
            Currency,
            Money,
            CncfTool,
            CncfToolCategory,
            CncfMaturity,
            DeploymentMode,
            ResourceRequirements,
            OpsHoursEstimate,
            ManagedServiceEquivalent,
            TcoCalculationRequest,
            TcoCalculationResult,
            CostRecommendation,
            InfrastructureCostComparison,
            InfraCompareRequest,
            cncf_client::CncfToolEnriched,
            cncf_client::CncfCategoryGroup,
            cncf_client::CncfToolsResponse,
            cncf_client::GitHubStats,
            cncf_client::ToolRecommendation,
            cncf_client::CategoryRecommendationResponse,
        ),
        responses(
            NotFoundResponse,
            BadRequestValidationResponse,
            BadRequestUuidResponse,
            InternalServerErrorResponse
        )
    ),
    tags(
        (name = entity::Model::TAG, description = "Cloud pricing management endpoints"),
        (name = "cncf", description = "CNCF tools and self-managed alternatives"),
        (name = "tco", description = "Total Cost of Ownership calculator")
    )
)]
pub struct ApiDoc;

/// Create the pricing router with all HTTP endpoints
pub fn router<R: PricingRepository + 'static>(service: PricingService<R>) -> Router {
    let shared_service = Arc::new(service);
    let tco_calculator = Arc::new(TcoCalculator::new(shared_service.clone()));

    Router::new()
        // Price CRUD endpoints
        .route("/", get(list_prices).post(create_price))
        .route("/{id}", get(get_price).put(update_price).delete(delete_price))
        .route("/compare", get(compare_prices))
        .route("/stats", get(get_price_stats))
        .with_state(shared_service)
        // CNCF tools endpoints (static data for TCO comparison)
        .route("/cncf/tools", get(list_cncf_tools))
        .route("/cncf/tools/{tool_id}", get(get_cncf_tool))
        // CNCF Landscape endpoints (real data from CNCF)
        .route("/cncf/landscape", get(list_cncf_landscape))
        .route("/cncf/landscape/{category}", get(get_cncf_category))
        .route("/cncf/recommend/{category}", get(get_cncf_recommendations))
        // TCO calculation endpoints
        .route("/tco/calculate", post(calculate_tco))
        .route("/tco/compare", post(compare_infrastructure))
        .with_state(tco_calculator)
}

/// List prices with optional filters
#[utoipa::path(
    get,
    path = "",
    tag = entity::Model::TAG,
    params(PriceFilter),
    responses(
        (status = 200, description = "List of prices", body = Vec<PriceEntry>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_prices<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    Query(filter): Query<PriceFilter>,
) -> PricingResult<Json<Vec<PriceEntry>>> {
    let prices = service.list(filter).await?;
    Ok(Json(prices))
}

/// Create a new price entry
#[utoipa::path(
    post,
    path = "",
    tag = entity::Model::TAG,
    request_body = CreatePriceEntry,
    responses(
        (status = 201, description = "Price entry created successfully", body = PriceEntry),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn create_price<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    ValidatedJson(input): ValidatedJson<CreatePriceEntry>,
) -> PricingResult<impl IntoResponse> {
    let price = service.create(input).await?;
    Ok((StatusCode::CREATED, Json(price)))
}

/// Get a price entry by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = entity::Model::TAG,
    params(
        ("id" = Uuid, Path, description = "Price entry ID")
    ),
    responses(
        (status = 200, description = "Price entry found", body = PriceEntry),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_price<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    UuidPath(id): UuidPath,
) -> PricingResult<Json<PriceEntry>> {
    let price = service.get_by_id(id).await?;
    Ok(Json(price))
}

/// Update a price entry
#[utoipa::path(
    put,
    path = "/{id}",
    tag = entity::Model::TAG,
    params(
        ("id" = Uuid, Path, description = "Price entry ID")
    ),
    request_body = UpdatePriceEntry,
    responses(
        (status = 200, description = "Price entry updated successfully", body = PriceEntry),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn update_price<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    UuidPath(id): UuidPath,
    ValidatedJson(input): ValidatedJson<UpdatePriceEntry>,
) -> PricingResult<Json<PriceEntry>> {
    let price = service.update(id, input).await?;
    Ok(Json(price))
}

/// Delete a price entry
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = entity::Model::TAG,
    params(
        ("id" = Uuid, Path, description = "Price entry ID")
    ),
    responses(
        (status = 204, description = "Price entry deleted successfully"),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn delete_price<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    UuidPath(id): UuidPath,
) -> PricingResult<impl IntoResponse> {
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Query parameters for price comparison
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct CompareQuery {
    /// Resource type to compare
    pub resource_type: ResourceType,
    /// Number of vCPUs (optional filter)
    pub vcpus: Option<i32>,
    /// Memory in GB (optional filter)
    pub memory_gb: Option<i32>,
    /// Regions to compare (comma-separated)
    pub regions: Option<String>,
    /// Providers to compare (comma-separated: aws,azure,gcp)
    pub providers: Option<String>,
}

/// Compare prices across cloud providers
#[utoipa::path(
    get,
    path = "/compare",
    tag = entity::Model::TAG,
    params(CompareQuery),
    responses(
        (status = 200, description = "Price comparison results", body = Vec<PriceComparison>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn compare_prices<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
    Query(query): Query<CompareQuery>,
) -> PricingResult<Json<Vec<PriceComparison>>> {
    let regions: Vec<String> = query
        .regions
        .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let providers: Vec<CloudProvider> = query
        .providers
        .map(|p| {
            p.split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect()
        })
        .unwrap_or_default();

    let comparisons = service
        .compare_prices(
            query.resource_type,
            query.vcpus,
            query.memory_gb,
            regions,
            providers,
        )
        .await?;

    Ok(Json(comparisons))
}

/// Price statistics response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PriceStats {
    /// Total number of price entries
    pub total_count: usize,
    /// Count by provider
    pub by_provider: ProviderCounts,
}

/// Count of prices per provider
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProviderCounts {
    pub aws: usize,
    pub azure: usize,
    pub gcp: usize,
}

/// Get pricing statistics
#[utoipa::path(
    get,
    path = "/stats",
    tag = entity::Model::TAG,
    responses(
        (status = 200, description = "Pricing statistics", body = PriceStats),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_price_stats<R: PricingRepository>(
    State(service): State<Arc<PricingService<R>>>,
) -> PricingResult<Json<PriceStats>> {
    let total_count = service.count().await?;
    let aws_count = service.count_by_provider(CloudProvider::Aws).await?;
    let azure_count = service.count_by_provider(CloudProvider::Azure).await?;
    let gcp_count = service.count_by_provider(CloudProvider::Gcp).await?;

    Ok(Json(PriceStats {
        total_count,
        by_provider: ProviderCounts {
            aws: aws_count,
            azure: azure_count,
            gcp: gcp_count,
        },
    }))
}

// =============================================================================
// CNCF Tools Endpoints
// =============================================================================

/// List all available CNCF tools
#[utoipa::path(
    get,
    path = "/cncf/tools",
    tag = "cncf",
    responses(
        (status = 200, description = "List of CNCF tools", body = Vec<CncfTool>)
    )
)]
async fn list_cncf_tools() -> Json<Vec<CncfTool>> {
    Json(get_cncf_tools())
}

/// Get a specific CNCF tool by ID
#[utoipa::path(
    get,
    path = "/cncf/tools/{tool_id}",
    tag = "cncf",
    params(
        ("tool_id" = String, Path, description = "Tool ID (e.g., 'cnpg', 'strimzi')")
    ),
    responses(
        (status = 200, description = "CNCF tool details", body = CncfTool),
        (status = 404, response = NotFoundResponse)
    )
)]
async fn get_cncf_tool(Path(tool_id): Path<String>) -> Result<Json<CncfTool>, (StatusCode, String)> {
    let tools = get_cncf_tools();
    tools
        .into_iter()
        .find(|t| t.id == tool_id)
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("CNCF tool '{}' not found", tool_id),
            )
        })
}

// =============================================================================
// CNCF Landscape (Real Data) Endpoints
// =============================================================================

use crate::ai_recommender::HeuristicRecommender;
use crate::cncf_client::CncfLandscapeClient;

/// Fetch all CNCF tools from the landscape, grouped by category
#[utoipa::path(
    get,
    path = "/cncf/landscape",
    tag = "cncf",
    responses(
        (status = 200, description = "CNCF tools grouped by category", body = cncf_client::CncfToolsResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_cncf_landscape() -> Result<Json<cncf_client::CncfToolsResponse>, (StatusCode, String)> {
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let client = CncfLandscapeClient::new(github_token);

    match client.fetch_cost_relevant_tools().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch CNCF landscape: {}", e),
        )),
    }
}

/// Get tools for a specific category with recommendations
#[utoipa::path(
    get,
    path = "/cncf/landscape/{category}",
    tag = "cncf",
    params(
        ("category" = String, Path, description = "Category (database, cache, message_queue, storage, observability, service_mesh, gitops)")
    ),
    responses(
        (status = 200, description = "Category tools with recommendations", body = cncf_client::CncfCategoryGroup),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_cncf_category(
    Path(category): Path<String>,
) -> Result<Json<cncf_client::CncfCategoryGroup>, (StatusCode, String)> {
    let target_category = match category.as_str() {
        "database" => CncfToolCategory::Database,
        "cache" => CncfToolCategory::Cache,
        "message_queue" => CncfToolCategory::MessageQueue,
        "storage" => CncfToolCategory::Storage,
        "observability" => CncfToolCategory::Observability,
        "service_mesh" => CncfToolCategory::ServiceMesh,
        "gitops" => CncfToolCategory::GitOps,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown category: {}. Valid: database, cache, message_queue, storage, observability, service_mesh, gitops", category),
            ))
        }
    };

    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let client = CncfLandscapeClient::new(github_token);

    match client.fetch_cost_relevant_tools().await {
        Ok(response) => {
            let category_group = response
                .categories
                .into_iter()
                .find(|c| c.category == target_category);

            match category_group {
                Some(group) => Ok(Json(group)),
                None => Err((
                    StatusCode::NOT_FOUND,
                    format!("No tools found for category: {}", category),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch CNCF landscape: {}", e),
        )),
    }
}

/// Get AI recommendations for tools in a category
#[utoipa::path(
    get,
    path = "/cncf/recommend/{category}",
    tag = "cncf",
    params(
        ("category" = String, Path, description = "Category to get recommendations for")
    ),
    responses(
        (status = 200, description = "AI recommendations for category", body = cncf_client::CategoryRecommendationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_cncf_recommendations(
    Path(category): Path<String>,
) -> Result<Json<cncf_client::CategoryRecommendationResponse>, (StatusCode, String)> {
    use crate::ai_recommender::AiRecommender;

    let target_category = match category.as_str() {
        "database" => CncfToolCategory::Database,
        "cache" => CncfToolCategory::Cache,
        "message_queue" => CncfToolCategory::MessageQueue,
        "storage" => CncfToolCategory::Storage,
        "observability" => CncfToolCategory::Observability,
        "service_mesh" => CncfToolCategory::ServiceMesh,
        "gitops" => CncfToolCategory::GitOps,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown category: {}", category),
            ))
        }
    };

    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let client = CncfLandscapeClient::new(github_token);

    let response = client.fetch_cost_relevant_tools().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch CNCF landscape: {}", e),
        )
    })?;

    let category_group = response
        .categories
        .into_iter()
        .find(|c| c.category == target_category)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("No tools found for category: {}", category),
            )
        })?;

    let recommender = HeuristicRecommender::new();
    let recommendations = recommender
        .recommend_for_category(target_category, &category_group.tools, None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate recommendations: {}", e),
            )
        })?;

    Ok(Json(recommendations))
}

// =============================================================================
// TCO Calculation Endpoints
// =============================================================================

/// Calculate TCO for a specific CNCF tool vs managed service
#[utoipa::path(
    post,
    path = "/tco/calculate",
    tag = "tco",
    request_body = TcoCalculationRequest,
    responses(
        (status = 200, description = "TCO calculation result", body = TcoCalculationResult),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn calculate_tco<R: PricingRepository>(
    State(calculator): State<Arc<TcoCalculator<R>>>,
    Json(request): Json<TcoCalculationRequest>,
) -> PricingResult<Json<TcoCalculationResult>> {
    let result = calculator.calculate_tco(request).await?;
    Ok(Json(result))
}

/// Request for comparing infrastructure costs
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct InfraCompareRequest {
    /// Cloud provider for pricing
    pub provider: CloudProvider,
    /// Region for pricing
    pub region: String,
    /// Deployment mode
    pub deployment_mode: DeploymentMode,
    /// Engineer hourly rate for ops cost calculation
    pub engineer_hourly_rate: Money,
    /// Number of similar workloads (amortize ops cost)
    pub workload_count: i32,
}

/// Compare infrastructure costs across all CNCF tools
#[utoipa::path(
    post,
    path = "/tco/compare",
    tag = "tco",
    request_body = InfraCompareRequest,
    responses(
        (status = 200, description = "Infrastructure cost comparison", body = InfrastructureCostComparison),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn compare_infrastructure<R: PricingRepository>(
    State(calculator): State<Arc<TcoCalculator<R>>>,
    Json(request): Json<InfraCompareRequest>,
) -> PricingResult<Json<InfrastructureCostComparison>> {
    let result = calculator
        .compare_infrastructure(
            request.provider,
            &request.region,
            request.deployment_mode,
            request.engineer_hourly_rate,
            request.workload_count,
        )
        .await?;
    Ok(Json(result))
}

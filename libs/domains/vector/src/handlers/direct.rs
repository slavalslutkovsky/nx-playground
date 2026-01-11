use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{VectorError, VectorResult};
use crate::models::{
    CreateCollection, EmbeddingModel, EmbeddingProviderType, TenantContext, VectorConfig,
};
use crate::repository::VectorRepository;
use crate::service::VectorService;

// Re-use HTTP types from grpc module
use super::grpc::{
    CollectionCreateRequest, CollectionResponse, CollectionsListResponse, DeleteResponse,
    ProjectIdParam, SearchRequest, SearchResponse, SearchResultItem, UpsertRequest, UpsertResponse,
    VectorGetResponse, VectorItem, VectorQueryParams,
};

// ===== Helper Functions =====

fn map_provider(provider: &Option<String>) -> EmbeddingProviderType {
    match provider.as_deref() {
        Some("vertexai") => EmbeddingProviderType::VertexAI,
        Some("cohere") => EmbeddingProviderType::Cohere,
        Some("voyage") => EmbeddingProviderType::Voyage,
        Some("anthropic") => EmbeddingProviderType::Anthropic,
        _ => EmbeddingProviderType::OpenAI,
    }
}

fn map_model(model: &Option<String>) -> EmbeddingModel {
    match model.as_deref() {
        Some("text-embedding-3-small") => EmbeddingModel::TextEmbedding3Small,
        Some("text-embedding-3-large") => EmbeddingModel::TextEmbedding3Large,
        Some("text-embedding-ada-002") => EmbeddingModel::TextEmbeddingAda002,
        Some("gecko") => EmbeddingModel::Gecko,
        Some("text-embedding-004") => EmbeddingModel::TextEmbedding004,
        Some("text-embedding-005") => EmbeddingModel::TextEmbedding005,
        _ => EmbeddingModel::TextEmbedding3Small,
    }
}

// ===== Handler Functions =====

/// Search vectors with automatic embedding (direct Qdrant)
#[utoipa::path(
    post,
    path = "/search",
    tag = "vectors-direct",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_vectors<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Json(req): Json<SearchRequest>,
) -> VectorResult<Json<SearchResponse>> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let tenant = TenantContext::new(project_id);
    let provider = map_provider(&req.provider);
    let model = map_model(&req.model);

    let results = service
        .search_with_embedding(
            &tenant,
            &req.collection,
            &req.query,
            req.limit,
            req.score_threshold,
            req.with_vectors,
            true, // with_payloads
            provider,
            model,
        )
        .await?;

    let result_items: Vec<SearchResultItem> = results
        .into_iter()
        .map(|r| SearchResultItem {
            id: r.id.to_string(),
            score: r.score,
            metadata: r.payload,
            vector: r.vector,
        })
        .collect();

    Ok(Json(SearchResponse {
        results: result_items,
        search_time_ms: 0,
    }))
}

/// Upsert a vector with automatic embedding (direct Qdrant)
#[utoipa::path(
    post,
    path = "/upsert",
    tag = "vectors-direct",
    request_body = UpsertRequest,
    responses(
        (status = 201, description = "Vector upserted successfully", body = UpsertResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn upsert_vector<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Json(req): Json<UpsertRequest>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;
    let vector_id = Uuid::parse_str(&req.id)
        .map_err(|_| VectorError::Validation("Invalid vector id".to_string()))?;

    let tenant = TenantContext::new(project_id);
    let provider = map_provider(&req.provider);
    let model = map_model(&req.model);

    let result_id = service
        .upsert_with_embedding(
            &tenant,
            &req.collection,
            vector_id,
            &req.content,
            req.metadata,
            provider,
            model,
            true, // wait
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(UpsertResponse {
            id: result_id.to_string(),
            success: true,
        }),
    ))
}

/// List collections for a project (direct Qdrant)
#[utoipa::path(
    get,
    path = "/collections",
    tag = "vectors-direct",
    params(
        ("project_id" = String, Query, description = "Project ID")
    ),
    responses(
        (status = 200, description = "List of collections", body = CollectionsListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_collections<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<Json<CollectionsListResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let tenant = TenantContext::new(project_id);
    let collections = service.list_collections(&tenant).await?;

    let collection_responses: Vec<CollectionResponse> = collections
        .into_iter()
        .map(|c| CollectionResponse {
            name: c.name,
            vectors_count: c.vectors_count,
            indexed_vectors_count: c.indexed_vectors_count,
            points_count: c.points_count,
            status: c.status.as_str().to_string(),
        })
        .collect();

    Ok(Json(CollectionsListResponse {
        collections: collection_responses,
    }))
}

/// Create a new collection (direct Qdrant)
#[utoipa::path(
    post,
    path = "/collections",
    tag = "vectors-direct",
    request_body = CollectionCreateRequest,
    responses(
        (status = 201, description = "Collection created", body = CollectionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_collection<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Json(req): Json<CollectionCreateRequest>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let tenant = TenantContext::new(project_id);

    let distance = match req.distance.as_deref() {
        Some("euclidean") => crate::models::DistanceMetric::Euclidean,
        Some("dot") => crate::models::DistanceMetric::DotProduct,
        _ => crate::models::DistanceMetric::Cosine,
    };

    let input = CreateCollection {
        name: req.name.clone(),
        config: VectorConfig::new(req.dimension).with_distance(distance),
    };

    let info = service.create_collection(&tenant, input).await?;

    Ok((
        StatusCode::CREATED,
        Json(CollectionResponse {
            name: info.name,
            vectors_count: info.vectors_count,
            indexed_vectors_count: info.indexed_vectors_count,
            points_count: info.points_count,
            status: info.status.as_str().to_string(),
        }),
    ))
}

/// Get collection info (direct Qdrant)
#[utoipa::path(
    get,
    path = "/collections/{name}",
    tag = "vectors-direct",
    params(
        ("name" = String, Path, description = "Collection name"),
        ("project_id" = String, Query, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Collection info", body = CollectionResponse),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_collection<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Path(name): Path<String>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<Json<CollectionResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let tenant = TenantContext::new(project_id);
    let info = service
        .get_collection(&tenant, &name)
        .await?
        .ok_or_else(|| VectorError::CollectionNotFound(name))?;

    Ok(Json(CollectionResponse {
        name: info.name,
        vectors_count: info.vectors_count,
        indexed_vectors_count: info.indexed_vectors_count,
        points_count: info.points_count,
        status: info.status.as_str().to_string(),
    }))
}

/// Delete a collection (direct Qdrant)
#[utoipa::path(
    delete,
    path = "/collections/{name}",
    tag = "vectors-direct",
    params(
        ("name" = String, Path, description = "Collection name"),
        ("project_id" = String, Query, description = "Project ID")
    ),
    responses(
        (status = 204, description = "Collection deleted"),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_collection<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Path(name): Path<String>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let tenant = TenantContext::new(project_id);
    service.delete_collection(&tenant, &name).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get vectors by IDs (direct Qdrant)
#[utoipa::path(
    get,
    path = "/vectors",
    tag = "vectors-direct",
    params(
        ("project_id" = String, Query, description = "Project ID"),
        ("collection" = String, Query, description = "Collection name"),
        ("ids" = String, Query, description = "Comma-separated vector IDs")
    ),
    responses(
        (status = 200, description = "Vector data", body = VectorGetResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_vectors<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Query(params): Query<VectorQueryParams>,
) -> VectorResult<Json<VectorGetResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let ids: Result<Vec<Uuid>, _> = params
        .ids
        .split(',')
        .map(|s| {
            Uuid::parse_str(s.trim())
                .map_err(|_| VectorError::Validation(format!("Invalid UUID: {}", s)))
        })
        .collect();
    let ids = ids?;

    let tenant = TenantContext::new(project_id);
    let vectors = service
        .get(&tenant, &params.collection, ids, true, true)
        .await?;

    let vector_items: Vec<VectorItem> = vectors
        .into_iter()
        .map(|v| VectorItem {
            id: v.id.to_string(),
            values: v.values,
            metadata: v.payload,
        })
        .collect();

    Ok(Json(VectorGetResponse {
        vectors: vector_items,
    }))
}

/// Delete vectors by IDs (direct Qdrant)
#[utoipa::path(
    delete,
    path = "/vectors",
    tag = "vectors-direct",
    params(
        ("project_id" = String, Query, description = "Project ID"),
        ("collection" = String, Query, description = "Collection name"),
        ("ids" = String, Query, description = "Comma-separated vector IDs")
    ),
    responses(
        (status = 200, description = "Vectors deleted", body = DeleteResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_vectors<R: VectorRepository>(
    State(service): State<Arc<VectorService<R>>>,
    Query(params): Query<VectorQueryParams>,
) -> VectorResult<Json<DeleteResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let ids: Result<Vec<Uuid>, _> = params
        .ids
        .split(',')
        .map(|s| {
            Uuid::parse_str(s.trim())
                .map_err(|_| VectorError::Validation(format!("Invalid UUID: {}", s)))
        })
        .collect();
    let ids = ids?;
    let count = ids.len() as u32;

    let tenant = TenantContext::new(project_id);
    service
        .delete(&tenant, &params.collection, ids, true)
        .await?;

    Ok(Json(DeleteResponse {
        deleted: count,
        success: true,
    }))
}

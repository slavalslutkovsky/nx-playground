use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use rpc::vector::vector_service_client::VectorServiceClient;
use rpc::vector::{
    CreateCollectionRequest as ProtoCreateCollection, DeleteCollectionRequest,
    DeleteRequest as ProtoDeleteRequest, EmbeddingModel as ProtoEmbeddingModel, EmbeddingProvider,
    GetCollectionRequest, GetRequest as ProtoGetRequest, ListCollectionsRequest,
    SearchWithEmbeddingRequest, UpsertWithEmbeddingRequest,
};
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::conversions::{bytes_to_uuid, uuid_to_bytes};
use crate::error::VectorError;

/// Error result type for vector handlers
pub type VectorResult<T> = Result<T, VectorError>;

// ===== HTTP Request/Response Types =====

/// Search request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// Project ID for multi-tenancy
    pub project_id: String,
    /// Collection name to search in
    pub collection: String,
    /// Search query text (will be embedded)
    pub query: String,
    /// Maximum number of results (default: 10)
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Minimum score threshold
    pub score_threshold: Option<f32>,
    /// Embedding provider (openai, vertexai, cohere, voyage)
    #[serde(default)]
    pub provider: Option<String>,
    /// Embedding model name
    #[serde(default)]
    pub model: Option<String>,
    /// Include vectors in response
    #[serde(default)]
    pub with_vectors: bool,
}

fn default_limit() -> u32 {
    10
}

/// Search response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub search_time_ms: u64,
}

/// Individual search result
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResultItem {
    pub id: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
}

/// Upsert request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertRequest {
    /// Project ID for multi-tenancy
    pub project_id: String,
    /// Collection name
    pub collection: String,
    /// Vector ID (UUID)
    pub id: String,
    /// Text content to embed
    pub content: String,
    /// Optional metadata payload
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Embedding provider
    #[serde(default)]
    pub provider: Option<String>,
    /// Embedding model
    #[serde(default)]
    pub model: Option<String>,
}

/// Upsert response
#[derive(Debug, Serialize, ToSchema)]
pub struct UpsertResponse {
    pub id: String,
    pub success: bool,
}

/// Collection create request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CollectionCreateRequest {
    /// Project ID for multi-tenancy
    pub project_id: String,
    /// Collection name
    pub name: String,
    /// Vector dimension (default: 1536 for OpenAI)
    #[serde(default = "default_dimension")]
    pub dimension: u32,
    /// Distance metric (cosine, euclidean, dot)
    #[serde(default)]
    pub distance: Option<String>,
}

fn default_dimension() -> u32 {
    1536
}

/// Collection info response
#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionResponse {
    pub name: String,
    pub vectors_count: u64,
    pub indexed_vectors_count: u64,
    pub points_count: u64,
    pub status: String,
}

/// List collections response
#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionsListResponse {
    pub collections: Vec<CollectionResponse>,
}

/// Query params for get/delete vectors
#[derive(Debug, Deserialize)]
pub struct VectorQueryParams {
    pub project_id: String,
    pub collection: String,
    /// Comma-separated list of UUIDs
    pub ids: String,
}

/// Get vectors response
#[derive(Debug, Serialize, ToSchema)]
pub struct VectorGetResponse {
    pub vectors: Vec<VectorItem>,
}

/// Individual vector item
#[derive(Debug, Serialize, ToSchema)]
pub struct VectorItem {
    pub id: String,
    pub values: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Delete response
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResponse {
    pub deleted: u32,
    pub success: bool,
}

// ===== Helper Functions =====

fn map_provider(provider: &Option<String>) -> i32 {
    match provider.as_deref() {
        Some("openai") => EmbeddingProvider::EmbeddingOpenai as i32,
        Some("vertexai") => EmbeddingProvider::EmbeddingVertexai as i32,
        Some("cohere") => EmbeddingProvider::EmbeddingCohere as i32,
        Some("voyage") => EmbeddingProvider::EmbeddingVoyage as i32,
        _ => EmbeddingProvider::EmbeddingOpenai as i32,
    }
}

fn map_model(model: &Option<String>) -> i32 {
    match model.as_deref() {
        Some("text-embedding-3-small") => ProtoEmbeddingModel::Embedding3Small as i32,
        Some("text-embedding-3-large") => ProtoEmbeddingModel::Embedding3Large as i32,
        Some("text-embedding-ada-002") => ProtoEmbeddingModel::EmbeddingAda002 as i32,
        Some("gecko") => ProtoEmbeddingModel::Gecko as i32,
        Some("text-embedding-004") => ProtoEmbeddingModel::TextEmbedding004 as i32,
        Some("text-embedding-005") => ProtoEmbeddingModel::TextEmbedding005 as i32,
        _ => ProtoEmbeddingModel::Embedding3Small as i32,
    }
}

// ===== Handler Functions =====

/// Search vectors with automatic embedding
#[utoipa::path(
    post,
    path = "/search",
    tag = "vectors",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_vectors(
    State(mut client): State<VectorServiceClient<Channel>>,
    Json(req): Json<SearchRequest>,
) -> VectorResult<Json<SearchResponse>> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let response = client
        .search_with_embedding(SearchWithEmbeddingRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: req.collection,
            text: req.query,
            limit: req.limit,
            score_threshold: req.score_threshold,
            with_payloads: true,
            with_vectors: req.with_vectors,
            provider: map_provider(&req.provider),
            model: map_model(&req.model),
            filter: None,
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    let inner = response.into_inner();
    let results = inner
        .results
        .into_iter()
        .map(|r| {
            let id = bytes_to_uuid(&r.id)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| "invalid".to_string());
            let metadata = r.payload.and_then(|p| {
                if p.json.is_empty() {
                    None
                } else {
                    serde_json::from_slice(&p.json).ok()
                }
            });
            let vector = r.vector.map(|v| v.values);

            SearchResultItem {
                id,
                score: r.score,
                metadata,
                vector,
            }
        })
        .collect();

    Ok(Json(SearchResponse {
        results,
        search_time_ms: inner.search_time_ms,
    }))
}

/// Upsert a vector with automatic embedding
#[utoipa::path(
    post,
    path = "/upsert",
    tag = "vectors",
    request_body = UpsertRequest,
    responses(
        (status = 201, description = "Vector upserted successfully", body = UpsertResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn upsert_vector(
    State(mut client): State<VectorServiceClient<Channel>>,
    Json(req): Json<UpsertRequest>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;
    let vector_id = Uuid::parse_str(&req.id)
        .map_err(|_| VectorError::Validation("Invalid vector id".to_string()))?;

    let payload = req.metadata.map(|m| rpc::vector::Payload {
        json: serde_json::to_vec(&m).unwrap_or_default(),
    });

    let response = client
        .upsert_with_embedding(UpsertWithEmbeddingRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: req.collection,
            id: uuid_to_bytes(vector_id),
            text: req.content,
            payload,
            provider: map_provider(&req.provider),
            model: map_model(&req.model),
            wait: true,
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    let inner = response.into_inner();
    let id = bytes_to_uuid(&inner.id)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| req.id);

    Ok((
        StatusCode::CREATED,
        Json(UpsertResponse { id, success: true }),
    ))
}

/// List collections for a project
#[utoipa::path(
    get,
    path = "/collections",
    tag = "vectors",
    params(
        ("project_id" = String, Query, description = "Project ID")
    ),
    responses(
        (status = 200, description = "List of collections", body = CollectionsListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_collections(
    State(mut client): State<VectorServiceClient<Channel>>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<Json<CollectionsListResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let response = client
        .list_collections(ListCollectionsRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    let collections = response
        .into_inner()
        .collections
        .into_iter()
        .map(|c| CollectionResponse {
            name: c.collection_name,
            vectors_count: c.vectors_count,
            indexed_vectors_count: c.indexed_vectors_count,
            points_count: c.points_count,
            status: c.status,
        })
        .collect();

    Ok(Json(CollectionsListResponse { collections }))
}

#[derive(Debug, Deserialize)]
pub struct ProjectIdParam {
    pub project_id: String,
}

/// Create a new collection
#[utoipa::path(
    post,
    path = "/collections",
    tag = "vectors",
    request_body = CollectionCreateRequest,
    responses(
        (status = 201, description = "Collection created", body = CollectionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_collection(
    State(mut client): State<VectorServiceClient<Channel>>,
    Json(req): Json<CollectionCreateRequest>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let distance = match req.distance.as_deref() {
        Some("euclidean") => rpc::vector::DistanceMetric::Euclidean as i32,
        Some("dot") => rpc::vector::DistanceMetric::DotProduct as i32,
        _ => rpc::vector::DistanceMetric::Cosine as i32,
    };

    let _response = client
        .create_collection(ProtoCreateCollection {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: req.name.clone(),
            config: Some(rpc::vector::VectorConfig {
                dimension: req.dimension,
                distance,
                hnsw: None,
            }),
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(CollectionResponse {
            name: req.name,
            vectors_count: 0,
            indexed_vectors_count: 0,
            points_count: 0,
            status: "green".to_string(),
        }),
    ))
}

/// Get collection info
#[utoipa::path(
    get,
    path = "/collections/{name}",
    tag = "vectors",
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
pub async fn get_collection(
    State(mut client): State<VectorServiceClient<Channel>>,
    Path(name): Path<String>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<Json<CollectionResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let response = client
        .get_collection(GetCollectionRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: name,
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                VectorError::CollectionNotFound(e.message().to_string())
            } else {
                VectorError::Internal(e.to_string())
            }
        })?;

    let info = response
        .into_inner()
        .info
        .ok_or_else(|| VectorError::Internal("Missing collection info".to_string()))?;

    Ok(Json(CollectionResponse {
        name: info.collection_name,
        vectors_count: info.vectors_count,
        indexed_vectors_count: info.indexed_vectors_count,
        points_count: info.points_count,
        status: info.status,
    }))
}

/// Delete a collection
#[utoipa::path(
    delete,
    path = "/collections/{name}",
    tag = "vectors",
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
pub async fn delete_collection(
    State(mut client): State<VectorServiceClient<Channel>>,
    Path(name): Path<String>,
    Query(params): Query<ProjectIdParam>,
) -> VectorResult<impl IntoResponse> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    client
        .delete_collection(DeleteCollectionRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: name,
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                VectorError::CollectionNotFound(e.message().to_string())
            } else {
                VectorError::Internal(e.to_string())
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get vectors by IDs
#[utoipa::path(
    get,
    path = "/vectors",
    tag = "vectors",
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
pub async fn get_vectors(
    State(mut client): State<VectorServiceClient<Channel>>,
    Query(params): Query<VectorQueryParams>,
) -> VectorResult<Json<VectorGetResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let ids: Result<Vec<Vec<u8>>, _> = params
        .ids
        .split(',')
        .map(|s| {
            Uuid::parse_str(s.trim())
                .map(uuid_to_bytes)
                .map_err(|_| VectorError::Validation(format!("Invalid UUID: {}", s)))
        })
        .collect();
    let ids = ids?;

    let response = client
        .get(ProtoGetRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: params.collection,
            ids,
            with_vectors: true,
            with_payloads: true,
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    let vectors = response
        .into_inner()
        .vectors
        .into_iter()
        .map(|v| {
            let id = bytes_to_uuid(&v.id)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| "invalid".to_string());
            let metadata = v.payload.and_then(|p| {
                if p.json.is_empty() {
                    None
                } else {
                    serde_json::from_slice(&p.json).ok()
                }
            });

            VectorItem {
                id,
                values: v.values,
                metadata,
            }
        })
        .collect();

    Ok(Json(VectorGetResponse { vectors }))
}

/// Delete vectors by IDs
#[utoipa::path(
    delete,
    path = "/vectors",
    tag = "vectors",
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
pub async fn delete_vectors(
    State(mut client): State<VectorServiceClient<Channel>>,
    Query(params): Query<VectorQueryParams>,
) -> VectorResult<Json<DeleteResponse>> {
    let project_id = Uuid::parse_str(&params.project_id)
        .map_err(|_| VectorError::Validation("Invalid project_id".to_string()))?;

    let ids: Result<Vec<Vec<u8>>, _> = params
        .ids
        .split(',')
        .map(|s| {
            Uuid::parse_str(s.trim())
                .map(uuid_to_bytes)
                .map_err(|_| VectorError::Validation(format!("Invalid UUID: {}", s)))
        })
        .collect();
    let ids = ids?;
    let count = ids.len() as u32;

    client
        .delete(ProtoDeleteRequest {
            tenant: Some(rpc::vector::TenantContext {
                project_id: uuid_to_bytes(project_id),
                namespace: None,
                user_id: None,
            }),
            collection_name: params.collection,
            ids,
            filter: None,
            wait: true,
        })
        .await
        .map_err(|e| VectorError::Internal(e.to_string()))?;

    Ok(Json(DeleteResponse {
        deleted: count,
        success: true,
    }))
}

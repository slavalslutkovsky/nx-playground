//! Qdrant vector database API endpoints.
//!
//! Provides endpoints for vector similarity search, collection management,
//! and RAG (Retrieval Augmented Generation) operations.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
};
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, DeleteCollectionBuilder, Distance, PointStruct,
        SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// Qdrant client state wrapper
#[derive(Clone)]
pub struct QdrantState {
    pub client: Arc<Qdrant>,
}

impl QdrantState {
    pub async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Qdrant::from_url(url).build()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

/// Document with embedding for vector storage
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Document {
    /// Unique document identifier
    pub id: Uuid,
    /// Document content/text
    pub content: String,
    /// Document metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Request to upsert documents with embeddings
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertRequest {
    /// Collection name
    pub collection: String,
    /// Documents to upsert
    pub documents: Vec<DocumentWithEmbedding>,
}

/// Document with its embedding vector
#[derive(Debug, Deserialize, ToSchema)]
pub struct DocumentWithEmbedding {
    /// Document data
    #[serde(flatten)]
    pub document: Document,
    /// Embedding vector (e.g., 1536 dimensions for OpenAI ada-002)
    pub embedding: Vec<f32>,
}

/// Vector search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// Collection to search in
    pub collection: String,
    /// Query embedding vector
    pub query_vector: Vec<f32>,
    /// Number of results to return
    #[serde(default = "default_limit")]
    pub limit: u64,
    /// Score threshold (0.0 - 1.0)
    #[serde(default)]
    pub score_threshold: Option<f32>,
    /// Filter by metadata
    #[serde(default)]
    pub filter: Option<std::collections::HashMap<String, String>>,
}

fn default_limit() -> u64 {
    10
}

/// Search result with score
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResult {
    /// Document ID
    pub id: Uuid,
    /// Similarity score
    pub score: f32,
    /// Document content
    pub content: String,
    /// Document metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Collection creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub name: String,
    /// Vector dimensions (e.g., 1536 for OpenAI, 768 for BERT)
    pub vector_size: u64,
    /// Distance metric
    #[serde(default)]
    pub distance: DistanceMetric,
}

/// Distance metric for vector similarity
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    #[default]
    Cosine,
    Euclidean,
    Dot,
}

impl From<DistanceMetric> for Distance {
    fn from(metric: DistanceMetric) -> Self {
        match metric {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        }
    }
}

/// Collection info response
#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Number of points in collection
    pub points_count: u64,
    /// Vector dimensions
    pub vector_size: u64,
}

/// API response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// List all collections
#[utoipa::path(
    get,
    path = "/collections",
    tag = "qdrant",
    responses(
        (status = 200, description = "List of collections", body = ApiResponse<Vec<String>>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_collections(
    State(state): State<QdrantState>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    match state.client.list_collections().await {
        Ok(collections) => {
            let names: Vec<String> = collections
                .collections
                .iter()
                .map(|c| c.name.clone())
                .collect();
            Ok(Json(ApiResponse::success(names)))
        }
        Err(e) => {
            error!("Failed to list collections: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create a new collection
#[utoipa::path(
    post,
    path = "/collections",
    tag = "qdrant",
    request_body = CreateCollectionRequest,
    responses(
        (status = 201, description = "Collection created", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_collection(
    State(state): State<QdrantState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StatusCode> {
    let result = state
        .client
        .create_collection(CreateCollectionBuilder::new(&req.name).vectors_config(
            VectorParamsBuilder::new(req.vector_size, req.distance.into()),
        ))
        .await;

    match result {
        Ok(_) => {
            info!("Created collection: {}", req.name);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success(format!(
                    "Collection '{}' created",
                    req.name
                ))),
            ))
        }
        Err(e) => {
            error!("Failed to create collection: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Delete a collection
#[utoipa::path(
    delete,
    path = "/collections/{name}",
    tag = "qdrant",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    responses(
        (status = 200, description = "Collection deleted", body = ApiResponse<String>),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_collection(
    State(state): State<QdrantState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state
        .client
        .delete_collection(DeleteCollectionBuilder::new(&name))
        .await
    {
        Ok(_) => {
            info!("Deleted collection: {}", name);
            Ok(Json(ApiResponse::success(format!(
                "Collection '{}' deleted",
                name
            ))))
        }
        Err(e) => {
            error!("Failed to delete collection: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Upsert documents with embeddings
#[utoipa::path(
    post,
    path = "/documents",
    tag = "qdrant",
    request_body = UpsertRequest,
    responses(
        (status = 200, description = "Documents upserted", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn upsert_documents(
    State(state): State<QdrantState>,
    Json(req): Json<UpsertRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let points: Vec<PointStruct> = req
        .documents
        .into_iter()
        .map(|doc| {
            let mut payload: std::collections::HashMap<String, serde_json::Value> =
                std::collections::HashMap::new();
            payload.insert(
                "content".to_string(),
                serde_json::Value::String(doc.document.content),
            );
            for (k, v) in doc.document.metadata {
                payload.insert(k, serde_json::Value::String(v));
            }

            PointStruct::new(doc.document.id.to_string(), doc.embedding, payload)
        })
        .collect();

    let count = points.len();

    match state
        .client
        .upsert_points(UpsertPointsBuilder::new(&req.collection, points))
        .await
    {
        Ok(_) => {
            info!("Upserted {} documents to {}", count, req.collection);
            Ok(Json(ApiResponse::success(format!(
                "Upserted {} documents",
                count
            ))))
        }
        Err(e) => {
            error!("Failed to upsert documents: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Search for similar documents
#[utoipa::path(
    post,
    path = "/search",
    tag = "qdrant",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = ApiResponse<Vec<SearchResult>>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search(
    State(state): State<QdrantState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ApiResponse<Vec<SearchResult>>>, StatusCode> {
    let mut search_builder =
        SearchPointsBuilder::new(&req.collection, req.query_vector, req.limit).with_payload(true);

    if let Some(threshold) = req.score_threshold {
        search_builder = search_builder.score_threshold(threshold);
    }

    match state.client.search_points(search_builder).await {
        Ok(response) => {
            let results: Vec<SearchResult> = response
                .result
                .into_iter()
                .filter_map(|point| {
                    // Extract point ID - handle both numeric and UUID formats
                    let id = point.id.as_ref().and_then(|pid| {
                        match &pid.point_id_options {
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => {
                                uuid.parse().ok()
                            }
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => {
                                // Create a deterministic UUID from the number
                                Some(Uuid::from_u128(*num as u128))
                            }
                            None => None,
                        }
                    })?;

                    let content = point
                        .payload
                        .get("content")
                        .and_then(|v| {
                            v.kind.as_ref().and_then(|k| {
                                if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or_default();

                    let metadata: std::collections::HashMap<String, String> = point
                        .payload
                        .iter()
                        .filter(|(k, _)| k.as_str() != "content")
                        .filter_map(|(k, v)| {
                            v.kind.as_ref().and_then(|kind| {
                                if let qdrant_client::qdrant::value::Kind::StringValue(s) = kind {
                                    Some((k.clone(), s.clone()))
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();

                    Some(SearchResult {
                        id,
                        score: point.score,
                        content,
                        metadata,
                    })
                })
                .collect();

            Ok(Json(ApiResponse::success(results)))
        }
        Err(e) => {
            error!("Search failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Health check for Qdrant connection
#[utoipa::path(
    get,
    path = "/health",
    tag = "qdrant",
    responses(
        (status = 200, description = "Qdrant is healthy", body = ApiResponse<String>),
        (status = 503, description = "Qdrant is unhealthy")
    )
)]
pub async fn health(
    State(state): State<QdrantState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.client.health_check().await {
        Ok(_) => Ok(Json(ApiResponse::success("Qdrant is healthy".to_string()))),
        Err(e) => {
            error!("Qdrant health check failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create Qdrant router with all endpoints
pub fn router(state: QdrantState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/{name}", delete(delete_collection))
        .route("/documents", post(upsert_documents))
        .route("/search", post(search))
        .with_state(state)
}

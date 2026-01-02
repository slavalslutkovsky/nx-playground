//! Milvus vector database API endpoints.
//!
//! Provides endpoints for vector similarity search and collection management
//! using Milvus REST API. This implementation uses HTTP client since the
//! Milvus Rust SDK is still maturing.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;

/// Milvus client state wrapper using REST API
#[derive(Clone)]
pub struct MilvusState {
    pub client: Arc<Client>,
    pub base_url: String,
}

impl MilvusState {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

/// Collection creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionRequest {
    /// Collection name
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Vector dimension
    pub dimension: u32,
    /// Metric type: L2, IP, COSINE
    #[serde(default, rename = "metricType")]
    pub metric_type: MetricType,
    /// Primary field name
    #[serde(default = "default_primary_field", rename = "primaryFieldName")]
    pub primary_field_name: String,
    /// Vector field name
    #[serde(default = "default_vector_field", rename = "vectorFieldName")]
    pub vector_field_name: String,
}

fn default_primary_field() -> String {
    "id".to_string()
}

fn default_vector_field() -> String {
    "vector".to_string()
}

/// Metric type for vector similarity
#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum MetricType {
    /// Euclidean distance
    L2,
    /// Inner product
    IP,
    /// Cosine similarity
    #[default]
    Cosine,
}

/// Insert vectors request
#[derive(Debug, Deserialize, ToSchema)]
pub struct InsertRequest {
    /// Collection name
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Data to insert (array of objects with id, vector, and optional fields)
    pub data: Vec<serde_json::Value>,
}

/// Vector search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// Collection name
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Query vectors
    pub data: Vec<Vec<f32>>,
    /// Number of results per query
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Output fields to return
    #[serde(default, rename = "outputFields")]
    pub output_fields: Vec<String>,
    /// Filter expression
    #[serde(default)]
    pub filter: Option<String>,
}

fn default_limit() -> u32 {
    10
}

/// Search result
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Distance/score
    pub distance: f32,
    /// Entity data
    #[serde(default)]
    pub entity: serde_json::Value,
}

/// Query request (filter-based retrieval)
#[derive(Debug, Deserialize, ToSchema)]
pub struct QueryRequest {
    /// Collection name
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Filter expression
    pub filter: String,
    /// Output fields
    #[serde(default, rename = "outputFields")]
    pub output_fields: Vec<String>,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Delete request
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteRequest {
    /// Collection name
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Filter expression or IDs
    pub filter: Option<String>,
    /// Specific IDs to delete
    pub ids: Option<Vec<String>>,
}

/// Collection info
#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Number of entities
    pub count: u64,
    /// Vector dimension
    pub dimension: u32,
    /// Is collection loaded
    pub loaded: bool,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            code: -1,
            data: None,
            message: Some(msg.into()),
        }
    }
}

/// Milvus REST API response wrapper
#[derive(Debug, Deserialize)]
struct MilvusResponse<T> {
    code: i32,
    data: Option<T>,
    message: Option<String>,
}

/// Health check for Milvus connection
#[utoipa::path(
    get,
    path = "/health",
    tag = "milvus",
    responses(
        (status = 200, description = "Milvus is healthy", body = ApiResponse<String>),
        (status = 503, description = "Milvus is unhealthy")
    )
)]
pub async fn health(
    State(state): State<MilvusState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let url = format!("{}/v2/vectordb/collections/list", state.base_url);

    match state.client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            Ok(Json(ApiResponse::success("Milvus is healthy".to_string())))
        }
        Ok(resp) => {
            let msg = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Milvus health check failed: {}", msg);
            Ok(Json(ApiResponse::error(msg)))
        }
        Err(e) => {
            error!("Milvus connection failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List all collections
#[utoipa::path(
    get,
    path = "/collections",
    tag = "milvus",
    responses(
        (status = 200, description = "List of collections", body = ApiResponse<Vec<String>>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_collections(
    State(state): State<MilvusState>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let url = format!("{}/v2/vectordb/collections/list", state.base_url);

    match state.client.get(&url).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<Vec<String>>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    Ok(Json(ApiResponse::success(
                        milvus_resp.data.unwrap_or_default(),
                    )))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
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
    tag = "milvus",
    request_body = CreateCollectionRequest,
    responses(
        (status = 201, description = "Collection created", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_collection(
    State(state): State<MilvusState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StatusCode> {
    let url = format!("{}/v2/vectordb/collections/create", state.base_url);

    let body = serde_json::json!({
        "collectionName": req.collection_name,
        "dimension": req.dimension,
        "metricType": req.metric_type,
        "primaryFieldName": req.primary_field_name,
        "vectorFieldName": req.vector_field_name,
    });

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    info!("Created collection: {}", req.collection_name);
                    Ok((
                        StatusCode::CREATED,
                        Json(ApiResponse::success(format!(
                            "Collection '{}' created",
                            req.collection_name
                        ))),
                    ))
                } else {
                    Ok((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::error(
                            milvus_resp
                                .message
                                .unwrap_or_else(|| "Unknown error".to_string()),
                        )),
                    ))
                }
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            )),
        },
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
    tag = "milvus",
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
    State(state): State<MilvusState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let url = format!("{}/v2/vectordb/collections/drop", state.base_url);

    let body = serde_json::json!({
        "collectionName": name,
    });

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    info!("Deleted collection: {}", name);
                    Ok(Json(ApiResponse::success(format!(
                        "Collection '{}' deleted",
                        name
                    ))))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Failed to delete collection: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get collection info
#[utoipa::path(
    get,
    path = "/collections/{name}",
    tag = "milvus",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    responses(
        (status = 200, description = "Collection info", body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_collection(
    State(state): State<MilvusState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let url = format!("{}/v2/vectordb/collections/describe", state.base_url);

    let body = serde_json::json!({
        "collectionName": name,
    });

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    Ok(Json(ApiResponse::success(
                        milvus_resp.data.unwrap_or_default(),
                    )))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Collection not found".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Failed to get collection: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Insert vectors into a collection
#[utoipa::path(
    post,
    path = "/vectors",
    tag = "milvus",
    request_body = InsertRequest,
    responses(
        (status = 200, description = "Vectors inserted", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn insert_vectors(
    State(state): State<MilvusState>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let url = format!("{}/v2/vectordb/entities/insert", state.base_url);

    let body = serde_json::json!({
        "collectionName": req.collection_name,
        "data": req.data,
    });

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    info!(
                        "Inserted {} vectors into {}",
                        req.data.len(),
                        req.collection_name
                    );
                    Ok(Json(ApiResponse::success(milvus_resp.data.unwrap_or(
                        serde_json::json!({
                            "insertCount": req.data.len()
                        }),
                    ))))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Failed to insert vectors: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Search for similar vectors
#[utoipa::path(
    post,
    path = "/search",
    tag = "milvus",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = ApiResponse<Vec<Vec<SearchResult>>>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search(
    State(state): State<MilvusState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let url = format!("{}/v2/vectordb/entities/search", state.base_url);

    let mut body = serde_json::json!({
        "collectionName": req.collection_name,
        "data": req.data,
        "limit": req.limit,
    });

    if !req.output_fields.is_empty() {
        body["outputFields"] = serde_json::json!(req.output_fields);
    }

    if let Some(filter) = req.filter {
        body["filter"] = serde_json::Value::String(filter);
    }

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    Ok(Json(ApiResponse::success(
                        milvus_resp.data.unwrap_or_default(),
                    )))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Search failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Query result wrapper (for OpenAPI schema)
#[derive(Debug, Serialize, ToSchema)]
pub struct QueryResult {
    /// Query results
    pub data: serde_json::Value,
}

/// Query entities by filter
#[utoipa::path(
    post,
    path = "/query",
    tag = "milvus",
    request_body = QueryRequest,
    responses(
        (status = 200, description = "Query results", body = ApiResponse<QueryResult>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn query(
    State(state): State<MilvusState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<ApiResponse<QueryResult>>, StatusCode> {
    let url = format!("{}/v2/vectordb/entities/query", state.base_url);

    let mut body = serde_json::json!({
        "collectionName": req.collection_name,
        "filter": req.filter,
        "limit": req.limit,
    });

    if !req.output_fields.is_empty() {
        body["outputFields"] = serde_json::json!(req.output_fields);
    }

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    Ok(Json(ApiResponse::success(QueryResult {
                        data: milvus_resp.data.unwrap_or_default(),
                    })))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Query failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete entities
#[utoipa::path(
    post,
    path = "/delete",
    tag = "milvus",
    request_body = DeleteRequest,
    responses(
        (status = 200, description = "Entities deleted", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_entities(
    State(state): State<MilvusState>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let url = format!("{}/v2/vectordb/entities/delete", state.base_url);

    let mut body = serde_json::json!({
        "collectionName": req.collection_name,
    });

    if let Some(filter) = req.filter {
        body["filter"] = serde_json::Value::String(filter);
    }

    if let Some(ids) = req.ids {
        body["ids"] = serde_json::json!(ids);
    }

    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => match resp.json::<MilvusResponse<serde_json::Value>>().await {
            Ok(milvus_resp) => {
                if milvus_resp.code == 0 {
                    info!("Deleted entities from {}", req.collection_name);
                    Ok(Json(ApiResponse::success(
                        milvus_resp.data.unwrap_or_default(),
                    )))
                } else {
                    Ok(Json(ApiResponse::error(
                        milvus_resp
                            .message
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    )))
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => {
            error!("Delete failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create Milvus router with all endpoints
pub fn router(state: MilvusState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/{name}", get(get_collection))
        .route("/collections/{name}", delete(delete_collection))
        .route("/vectors", post(insert_vectors))
        .route("/search", post(search))
        .route("/query", post(query))
        .route("/delete", post(delete_entities))
        .with_state(state)
}

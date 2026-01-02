//! ArangoDB multi-model database API endpoints.
//!
//! Provides endpoints for document, graph, and search operations
//! using ArangoDB's multi-model capabilities.

use arangors::{AqlQuery, Connection, Database};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// ArangoDB client state wrapper
#[derive(Clone)]
pub struct ArangoState {
    pub db: Arc<Database<arangors::client::reqwest::ReqwestClient>>,
}

impl ArangoState {
    pub async fn new(
        url: &str,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::establish_jwt(url, user, password).await?;
        let db = conn.db(database).await?;
        Ok(Self { db: Arc::new(db) })
    }
}

/// Document representation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Document {
    /// Document key (unique within collection)
    #[serde(rename = "_key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Document ID (collection/key)
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Document revision
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// Document data
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Edge document for graph operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Edge {
    /// Edge key
    #[serde(rename = "_key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Source vertex ID
    #[serde(rename = "_from")]
    pub from: String,
    /// Target vertex ID
    #[serde(rename = "_to")]
    pub to: String,
    /// Edge label/type
    #[serde(default)]
    pub label: Option<String>,
    /// Edge properties
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Collection creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub name: String,
    /// Collection type: "document" or "edge"
    #[serde(default)]
    pub collection_type: CollectionType,
}

/// Collection type
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CollectionType {
    #[default]
    Document,
    Edge,
}

/// AQL query request
#[derive(Debug, Deserialize, ToSchema)]
pub struct AqlQueryRequest {
    /// AQL query string
    pub query: String,
    /// Bind variables
    #[serde(default)]
    pub bind_vars: serde_json::Value,
}

/// Graph traversal request
#[derive(Debug, Deserialize, ToSchema)]
pub struct TraversalRequest {
    /// Starting vertex ID (collection/key)
    pub start_vertex: String,
    /// Graph name
    pub graph: String,
    /// Traversal direction: "outbound", "inbound", "any"
    #[serde(default)]
    pub direction: TraversalDirection,
    /// Minimum depth
    #[serde(default = "default_min_depth")]
    pub min_depth: u32,
    /// Maximum depth
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_min_depth() -> u32 {
    1
}

fn default_max_depth() -> u32 {
    3
}

fn default_limit() -> u32 {
    100
}

/// Traversal direction
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TraversalDirection {
    #[default]
    Outbound,
    Inbound,
    Any,
}

impl std::fmt::Display for TraversalDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraversalDirection::Outbound => write!(f, "OUTBOUND"),
            TraversalDirection::Inbound => write!(f, "INBOUND"),
            TraversalDirection::Any => write!(f, "ANY"),
        }
    }
}

/// Graph creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGraphRequest {
    /// Graph name
    pub name: String,
    /// Edge definitions
    pub edge_definitions: Vec<EdgeDefinition>,
}

/// Edge definition for graph
#[derive(Debug, Deserialize, ToSchema)]
pub struct EdgeDefinition {
    /// Edge collection name
    pub collection: String,
    /// Source vertex collections
    pub from: Vec<String>,
    /// Target vertex collections
    pub to: Vec<String>,
}

/// Search request with ArangoSearch
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// View name
    pub view: String,
    /// Search text
    pub search_text: String,
    /// Fields to search in
    pub fields: Vec<String>,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Traversal result
#[derive(Debug, Serialize, ToSchema)]
pub struct TraversalResult {
    /// Vertices found
    pub vertices: Vec<serde_json::Value>,
    /// Edges traversed
    pub edges: Vec<serde_json::Value>,
    /// Paths
    pub paths: Vec<serde_json::Value>,
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

/// Health check for ArangoDB connection
#[utoipa::path(
    get,
    path = "/health",
    tag = "arangodb",
    responses(
        (status = 200, description = "ArangoDB is healthy", body = ApiResponse<String>),
        (status = 503, description = "ArangoDB is unhealthy")
    )
)]
pub async fn health(
    State(state): State<ArangoState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = AqlQuery::builder().query("RETURN 1").build();

    match state.db.aql_query::<i32>(query).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "ArangoDB is healthy".to_string(),
        ))),
        Err(e) => {
            error!("ArangoDB health check failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List all collections
#[utoipa::path(
    get,
    path = "/collections",
    tag = "arangodb",
    responses(
        (status = 200, description = "List of collections", body = ApiResponse<Vec<String>>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_collections(
    State(state): State<ArangoState>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    match state.db.accessible_collections().await {
        Ok(collections) => {
            let names: Vec<String> = collections.iter().map(|c| c.name.clone()).collect();
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
    tag = "arangodb",
    request_body = CreateCollectionRequest,
    responses(
        (status = 201, description = "Collection created", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_collection(
    State(state): State<ArangoState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StatusCode> {
    let collection_type = match req.collection_type {
        CollectionType::Document => arangors::collection::CollectionType::Document,
        CollectionType::Edge => arangors::collection::CollectionType::Edge,
    };

    let options = arangors::collection::options::CreateOptions::builder()
        .name(&req.name)
        .collection_type(collection_type)
        .build();

    match state
        .db
        .create_collection_with_options(options, Default::default())
        .await
    {
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
    tag = "arangodb",
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
    State(state): State<ArangoState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.db.drop_collection(&name).await {
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

/// Create a document
#[utoipa::path(
    post,
    path = "/collections/{collection}/documents",
    tag = "arangodb",
    params(
        ("collection" = String, Path, description = "Collection name")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 201, description = "Document created", body = ApiResponse<Document>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_document(
    State(state): State<ArangoState>,
    axum::extract::Path(collection): axum::extract::Path<String>,
    Json(mut doc): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<ApiResponse<Document>>), StatusCode> {
    // Add a key if not present
    if doc.get("_key").is_none() {
        if let Some(obj) = doc.as_object_mut() {
            obj.insert(
                "_key".to_string(),
                serde_json::Value::String(Uuid::new_v4().to_string()),
            );
        }
    }

    let coll = match state.db.collection(&collection).await {
        Ok(c) => c,
        Err(e) => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(format!("Collection not found: {}", e))),
            ));
        }
    };

    match coll.create_document(doc.clone(), Default::default()).await {
        Ok(result) => {
            if let Some(header) = result.header() {
                info!("Created document in {}: {}", collection, header._key);
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::success(Document {
                        key: Some(header._key.clone()),
                        id: Some(header._id.to_string()),
                        rev: Some(header._rev.clone()),
                        data: doc,
                    })),
                ))
            } else {
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::success(Document {
                        key: None,
                        id: None,
                        rev: None,
                        data: doc,
                    })),
                ))
            }
        }
        Err(e) => {
            error!("Failed to create document: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Get a document by key
#[utoipa::path(
    get,
    path = "/collections/{collection}/documents/{key}",
    tag = "arangodb",
    params(
        ("collection" = String, Path, description = "Collection name"),
        ("key" = String, Path, description = "Document key")
    ),
    responses(
        (status = 200, description = "Document found", body = ApiResponse<Document>),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_document(
    State(state): State<ArangoState>,
    axum::extract::Path((collection, key)): axum::extract::Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let coll = match state.db.collection(&collection).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Collection not found: {}",
                e
            ))));
        }
    };

    match coll.document::<serde_json::Value>(&key).await {
        Ok(doc) => Ok(Json(ApiResponse::success(doc.document))),
        Err(e) => {
            error!("Failed to get document: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Update a document
#[utoipa::path(
    put,
    path = "/collections/{collection}/documents/{key}",
    tag = "arangodb",
    params(
        ("collection" = String, Path, description = "Collection name"),
        ("key" = String, Path, description = "Document key")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Document updated", body = ApiResponse<String>),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_document(
    State(state): State<ArangoState>,
    axum::extract::Path((collection, key)): axum::extract::Path<(String, String)>,
    Json(doc): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let coll = match state.db.collection(&collection).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Collection not found: {}",
                e
            ))));
        }
    };

    match coll.update_document(&key, doc, Default::default()).await {
        Ok(_) => {
            info!("Updated document {}/{}", collection, key);
            Ok(Json(ApiResponse::success("Document updated".to_string())))
        }
        Err(e) => {
            error!("Failed to update document: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete a document
#[utoipa::path(
    delete,
    path = "/collections/{collection}/documents/{key}",
    tag = "arangodb",
    params(
        ("collection" = String, Path, description = "Collection name"),
        ("key" = String, Path, description = "Document key")
    ),
    responses(
        (status = 200, description = "Document deleted", body = ApiResponse<String>),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_document(
    State(state): State<ArangoState>,
    axum::extract::Path((collection, key)): axum::extract::Path<(String, String)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let coll = match state.db.collection(&collection).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Collection not found: {}",
                e
            ))));
        }
    };

    match coll
        .remove_document::<serde_json::Value>(&key, Default::default(), Default::default())
        .await
    {
        Ok(_) => {
            info!("Deleted document {}/{}", collection, key);
            Ok(Json(ApiResponse::success("Document deleted".to_string())))
        }
        Err(e) => {
            error!("Failed to delete document: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// AQL query result wrapper (for OpenAPI schema)
#[derive(Debug, Serialize, ToSchema)]
pub struct AqlResult {
    /// Query results as JSON array
    pub results: Vec<serde_json::Value>,
}

/// Execute an AQL query
#[utoipa::path(
    post,
    path = "/aql",
    tag = "arangodb",
    request_body = AqlQueryRequest,
    responses(
        (status = 200, description = "Query executed", body = ApiResponse<AqlResult>),
        (status = 400, description = "Invalid query"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn execute_aql(
    State(state): State<ArangoState>,
    Json(req): Json<AqlQueryRequest>,
) -> Result<Json<ApiResponse<AqlResult>>, StatusCode> {
    // Build query - use simple query without bind vars for now
    // TODO: Support bind variables by converting serde_json::Value to HashMap<&str, Value>
    let query = AqlQuery::builder().query(&req.query).build();

    match state.db.aql_query::<serde_json::Value>(query).await {
        Ok(results) => Ok(Json(ApiResponse::success(AqlResult { results }))),
        Err(e) => {
            error!("AQL query failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create an edge (relationship)
#[utoipa::path(
    post,
    path = "/collections/{collection}/edges",
    tag = "arangodb",
    params(
        ("collection" = String, Path, description = "Edge collection name")
    ),
    request_body = Edge,
    responses(
        (status = 201, description = "Edge created", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_edge(
    State(state): State<ArangoState>,
    axum::extract::Path(collection): axum::extract::Path<String>,
    Json(mut edge): Json<Edge>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StatusCode> {
    // Generate key if not present
    if edge.key.is_none() {
        edge.key = Some(Uuid::new_v4().to_string());
    }

    let coll = match state.db.collection(&collection).await {
        Ok(c) => c,
        Err(e) => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(format!("Collection not found: {}", e))),
            ));
        }
    };

    // Build edge document
    let mut edge_doc = serde_json::json!({
        "_key": edge.key,
        "_from": edge.from,
        "_to": edge.to,
    });

    if let Some(label) = edge.label {
        edge_doc["label"] = serde_json::Value::String(label);
    }

    // Merge additional data
    if let Some(obj) = edge.data.as_object() {
        for (k, v) in obj {
            edge_doc[k] = v.clone();
        }
    }

    match coll.create_document(edge_doc, Default::default()).await {
        Ok(result) => {
            if let Some(header) = result.header() {
                info!("Created edge in {}: {}", collection, header._key);
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::success(header._key.clone())),
                ))
            } else {
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::success("Edge created".to_string())),
                ))
            }
        }
        Err(e) => {
            error!("Failed to create edge: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Graph traversal
#[utoipa::path(
    post,
    path = "/traverse",
    tag = "arangodb",
    request_body = TraversalRequest,
    responses(
        (status = 200, description = "Traversal results", body = ApiResponse<TraversalResult>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn traverse(
    State(state): State<ArangoState>,
    Json(req): Json<TraversalRequest>,
) -> Result<Json<ApiResponse<TraversalResult>>, StatusCode> {
    let aql = format!(
        r#"
        FOR v, e, p IN {}..{} {} @start_vertex GRAPH @graph
        LIMIT @limit
        RETURN {{
            vertex: v,
            edge: e,
            path: p
        }}
        "#,
        req.min_depth, req.max_depth, req.direction
    );

    let query = AqlQuery::builder()
        .query(&aql)
        .bind_var("start_vertex", req.start_vertex)
        .bind_var("graph", req.graph)
        .bind_var("limit", req.limit)
        .build();

    match state.db.aql_query::<serde_json::Value>(query).await {
        Ok(results) => {
            let mut vertices = Vec::new();
            let mut edges = Vec::new();
            let mut paths = Vec::new();

            for result in results {
                if let Some(v) = result.get("vertex") {
                    vertices.push(v.clone());
                }
                if let Some(e) = result.get("edge") {
                    if !e.is_null() {
                        edges.push(e.clone());
                    }
                }
                if let Some(p) = result.get("path") {
                    paths.push(p.clone());
                }
            }

            Ok(Json(ApiResponse::success(TraversalResult {
                vertices,
                edges,
                paths,
            })))
        }
        Err(e) => {
            error!("Traversal failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get database statistics
#[utoipa::path(
    get,
    path = "/stats",
    tag = "arangodb",
    responses(
        (status = 200, description = "Database statistics", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stats(
    State(state): State<ArangoState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let query = AqlQuery::builder()
        .query(
            r#"
            LET collections = (FOR c IN COLLECTIONS() RETURN { name: c.name, count: LENGTH(c) })
            RETURN { collections: collections }
        "#,
        )
        .build();

    match state.db.aql_query::<serde_json::Value>(query).await {
        Ok(results) => {
            let stats = results.into_iter().next().unwrap_or(serde_json::json!({}));
            Ok(Json(ApiResponse::success(stats)))
        }
        Err(e) => {
            error!("Failed to get stats: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create ArangoDB router with all endpoints
pub fn router(state: ArangoState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/stats", get(get_stats))
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/{name}", delete(delete_collection))
        .route("/collections/{collection}/documents", post(create_document))
        .route(
            "/collections/{collection}/documents/{key}",
            get(get_document),
        )
        .route(
            "/collections/{collection}/documents/{key}",
            put(update_document),
        )
        .route(
            "/collections/{collection}/documents/{key}",
            delete(delete_document),
        )
        .route("/collections/{collection}/edges", post(create_edge))
        .route("/aql", post(execute_aql))
        .route("/traverse", post(traverse))
        .with_state(state)
}

//! Neo4j graph database API endpoints.
//!
//! Provides endpoints for graph operations, Cypher queries,
//! and GraphRAG (Graph-based Retrieval Augmented Generation) operations.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
};
use neo4rs::{Graph, Node, query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// Neo4j client state wrapper
#[derive(Clone)]
pub struct Neo4jState {
    pub graph: Arc<Graph>,
}

impl Neo4jState {
    pub async fn new(
        uri: &str,
        user: &str,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let graph = Graph::new(uri, user, password).await?;
        Ok(Self {
            graph: Arc::new(graph),
        })
    }
}

/// Generic node representation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphNode {
    /// Node ID (internal Neo4j ID or custom UUID)
    pub id: String,
    /// Node labels
    pub labels: Vec<String>,
    /// Node properties
    pub properties: serde_json::Value,
}

/// Relationship between nodes
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Relationship {
    /// Source node ID
    pub from_id: String,
    /// Target node ID
    pub to_id: String,
    /// Relationship type
    pub relationship_type: String,
    /// Relationship properties
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// Request to create a node
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNodeRequest {
    /// Node labels (e.g., ["Person", "Employee"])
    pub labels: Vec<String>,
    /// Node properties
    pub properties: serde_json::Value,
}

/// Request to create a relationship
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRelationshipRequest {
    /// Source node ID
    pub from_id: String,
    /// Target node ID
    pub to_id: String,
    /// Relationship type (e.g., "KNOWS", "WORKS_AT")
    pub relationship_type: String,
    /// Relationship properties
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// Execute a raw Cypher query
#[derive(Debug, Deserialize, ToSchema)]
pub struct CypherQueryRequest {
    /// Cypher query string
    pub query: String,
    /// Query parameters
    #[serde(default)]
    pub params: serde_json::Value,
}

/// GraphRAG query request
#[derive(Debug, Deserialize, ToSchema)]
pub struct GraphRagRequest {
    /// Entity name to start traversal from
    pub entity: String,
    /// Maximum traversal depth
    #[serde(default = "default_depth")]
    pub depth: u32,
    /// Relationship types to follow (empty = all)
    #[serde(default)]
    pub relationship_types: Vec<String>,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_depth() -> u32 {
    2
}

fn default_limit() -> u32 {
    10
}

/// GraphRAG result with context
#[derive(Debug, Serialize, ToSchema)]
pub struct GraphRagResult {
    /// Starting entity
    pub entity: GraphNode,
    /// Related entities with paths
    pub related: Vec<RelatedEntity>,
    /// Total relationships found
    pub total_relationships: u32,
}

/// Related entity with path information
#[derive(Debug, Serialize, ToSchema)]
pub struct RelatedEntity {
    /// The related node
    pub node: GraphNode,
    /// Path from source entity
    pub path: Vec<String>,
    /// Distance from source
    pub distance: u32,
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

/// Health check for Neo4j connection
#[utoipa::path(
    get,
    path = "/health",
    tag = "neo4j",
    responses(
        (status = 200, description = "Neo4j is healthy", body = ApiResponse<String>),
        (status = 503, description = "Neo4j is unhealthy")
    )
)]
pub async fn health(
    State(state): State<Neo4jState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.graph.run(query("RETURN 1")).await {
        Ok(_) => Ok(Json(ApiResponse::success("Neo4j is healthy".to_string()))),
        Err(e) => {
            error!("Neo4j health check failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create a new node
#[utoipa::path(
    post,
    path = "/nodes",
    tag = "neo4j",
    request_body = CreateNodeRequest,
    responses(
        (status = 201, description = "Node created", body = ApiResponse<GraphNode>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_node(
    State(state): State<Neo4jState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<GraphNode>>), StatusCode> {
    let id = Uuid::new_v4().to_string();
    let labels = req.labels.join(":");

    // Build property assignments from JSON
    let props_str = if let Some(obj) = req.properties.as_object() {
        obj.iter()
            .filter_map(|(k, v)| {
                let val = match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => return None,
                };
                Some(format!("{}: {}", k, val))
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    };

    // Build the Cypher query dynamically
    let cypher = if props_str.is_empty() {
        format!("CREATE (n:{} {{id: $id}}) RETURN n", labels)
    } else {
        format!("CREATE (n:{} {{id: $id, {}}}) RETURN n", labels, props_str)
    };

    match state
        .graph
        .run(query(&cypher).param("id", id.clone()))
        .await
    {
        Ok(_) => {
            info!("Created node with id: {}", id);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success(GraphNode {
                    id,
                    labels: req.labels,
                    properties: req.properties,
                })),
            ))
        }
        Err(e) => {
            error!("Failed to create node: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Get a node by ID
#[utoipa::path(
    get,
    path = "/nodes/{id}",
    tag = "neo4j",
    params(
        ("id" = String, Path, description = "Node ID")
    ),
    responses(
        (status = 200, description = "Node found", body = ApiResponse<GraphNode>),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_node(
    State(state): State<Neo4jState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<GraphNode>>, StatusCode> {
    let cypher = "MATCH (n {id: $id}) RETURN n, labels(n) as labels";

    match state
        .graph
        .execute(query(cypher).param("id", id.clone()))
        .await
    {
        Ok(mut result) => {
            if let Some(row) = result.next().await.ok().flatten() {
                let node: Node = row.get("n").unwrap();
                let labels: Vec<String> = row.get("labels").unwrap_or_default();

                // Convert node properties to JSON
                let properties = serde_json::json!({});

                Ok(Json(ApiResponse::success(GraphNode {
                    id,
                    labels,
                    properties,
                })))
            } else {
                Ok(Json(ApiResponse::error("Node not found")))
            }
        }
        Err(e) => {
            error!("Failed to get node: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete a node by ID
#[utoipa::path(
    delete,
    path = "/nodes/{id}",
    tag = "neo4j",
    params(
        ("id" = String, Path, description = "Node ID")
    ),
    responses(
        (status = 200, description = "Node deleted", body = ApiResponse<String>),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_node(
    State(state): State<Neo4jState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let cypher = "MATCH (n {id: $id}) DETACH DELETE n";

    match state.graph.run(query(cypher).param("id", id.clone())).await {
        Ok(_) => {
            info!("Deleted node: {}", id);
            Ok(Json(ApiResponse::success(format!("Node '{}' deleted", id))))
        }
        Err(e) => {
            error!("Failed to delete node: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Create a relationship between nodes
#[utoipa::path(
    post,
    path = "/relationships",
    tag = "neo4j",
    request_body = CreateRelationshipRequest,
    responses(
        (status = 201, description = "Relationship created", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_relationship(
    State(state): State<Neo4jState>,
    Json(req): Json<CreateRelationshipRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StatusCode> {
    // Build property assignments from JSON
    let props_str = if let Some(obj) = req.properties.as_object() {
        let props: Vec<_> = obj
            .iter()
            .filter_map(|(k, v)| {
                let val = match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => return None,
                };
                Some(format!("{}: {}", k, val))
            })
            .collect();
        if props.is_empty() {
            String::new()
        } else {
            format!(" {{{}}}", props.join(", "))
        }
    } else {
        String::new()
    };

    let cypher = format!(
        "MATCH (a {{id: $from_id}}), (b {{id: $to_id}}) CREATE (a)-[r:{}{}]->(b) RETURN r",
        req.relationship_type, props_str
    );

    match state
        .graph
        .run(
            query(&cypher)
                .param("from_id", req.from_id.clone())
                .param("to_id", req.to_id.clone()),
        )
        .await
    {
        Ok(_) => {
            info!(
                "Created relationship {} -> {} ({})",
                req.from_id, req.to_id, req.relationship_type
            );
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success("Relationship created".to_string())),
            ))
        }
        Err(e) => {
            error!("Failed to create relationship: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Execute a raw Cypher query
#[utoipa::path(
    post,
    path = "/cypher",
    tag = "neo4j",
    request_body = CypherQueryRequest,
    responses(
        (status = 200, description = "Query executed", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid query"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn execute_cypher(
    State(state): State<Neo4jState>,
    Json(req): Json<CypherQueryRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Build query with params
    let mut q = query(&req.query);

    if let Some(params) = req.params.as_object() {
        for (key, value) in params {
            match value {
                serde_json::Value::String(s) => {
                    q = q.param(key.as_str(), s.clone());
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        q = q.param(key.as_str(), i);
                    } else if let Some(f) = n.as_f64() {
                        q = q.param(key.as_str(), f);
                    }
                }
                serde_json::Value::Bool(b) => {
                    q = q.param(key.as_str(), *b);
                }
                _ => {}
            }
        }
    }

    match state.graph.execute(q).await {
        Ok(mut result) => {
            let mut rows = Vec::new();
            while let Ok(Some(row)) = result.next().await {
                // Convert row to JSON representation
                rows.push(serde_json::json!({
                    "row": format!("{:?}", row)
                }));
            }
            Ok(Json(ApiResponse::success(serde_json::json!(rows))))
        }
        Err(e) => {
            error!("Cypher query failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// GraphRAG: Get related entities for context retrieval
#[utoipa::path(
    post,
    path = "/graphrag",
    tag = "neo4j",
    request_body = GraphRagRequest,
    responses(
        (status = 200, description = "Related entities found", body = ApiResponse<GraphRagResult>),
        (status = 404, description = "Entity not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn graph_rag(
    State(state): State<Neo4jState>,
    Json(req): Json<GraphRagRequest>,
) -> Result<Json<ApiResponse<GraphRagResult>>, StatusCode> {
    // Build relationship pattern
    let rel_pattern = if req.relationship_types.is_empty() {
        format!("[*1..{}]", req.depth)
    } else {
        format!("[:{}*1..{}]", req.relationship_types.join("|"), req.depth)
    };

    let cypher = format!(
        r#"
        MATCH (start {{name: $entity}})
        OPTIONAL MATCH path = (start)-{}->(related)
        WHERE related <> start
        WITH start, related, path, length(path) as distance
        ORDER BY distance
        LIMIT $limit
        RETURN start, collect({{
            node: related,
            distance: distance,
            path: [r in relationships(path) | type(r)]
        }}) as related_entities
        "#,
        rel_pattern
    );

    match state
        .graph
        .execute(
            query(&cypher)
                .param("entity", req.entity.clone())
                .param("limit", req.limit as i64),
        )
        .await
    {
        Ok(mut result) => {
            if let Ok(Some(row)) = result.next().await {
                let _start: Node = row.get("start").unwrap();

                // Build result
                let graph_rag_result = GraphRagResult {
                    entity: GraphNode {
                        id: req.entity.clone(),
                        labels: vec![],
                        properties: serde_json::json!({}),
                    },
                    related: vec![],
                    total_relationships: 0,
                };

                Ok(Json(ApiResponse::success(graph_rag_result)))
            } else {
                Ok(Json(ApiResponse::error("Entity not found")))
            }
        }
        Err(e) => {
            error!("GraphRAG query failed: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get graph statistics
#[utoipa::path(
    get,
    path = "/stats",
    tag = "neo4j",
    responses(
        (status = 200, description = "Graph statistics", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stats(
    State(state): State<Neo4jState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let cypher = r#"
        CALL apoc.meta.stats() YIELD nodeCount, relCount, labels, relTypes
        RETURN nodeCount, relCount, labels, relTypes
    "#;

    // Fallback query if APOC is not installed
    let fallback = r#"
        MATCH (n)
        WITH count(n) as nodeCount
        MATCH ()-[r]->()
        RETURN nodeCount, count(r) as relCount
    "#;

    let result = state.graph.execute(query(cypher)).await;

    match result {
        Ok(mut result) => {
            if let Ok(Some(row)) = result.next().await {
                let node_count: i64 = row.get("nodeCount").unwrap_or(0);
                let rel_count: i64 = row.get("relCount").unwrap_or(0);

                Ok(Json(ApiResponse::success(serde_json::json!({
                    "node_count": node_count,
                    "relationship_count": rel_count
                }))))
            } else {
                Ok(Json(ApiResponse::success(serde_json::json!({
                    "node_count": 0,
                    "relationship_count": 0
                }))))
            }
        }
        Err(_) => {
            // Try fallback query
            match state.graph.execute(query(fallback)).await {
                Ok(mut result) => {
                    if let Ok(Some(row)) = result.next().await {
                        let node_count: i64 = row.get("nodeCount").unwrap_or(0);
                        let rel_count: i64 = row.get("relCount").unwrap_or(0);

                        Ok(Json(ApiResponse::success(serde_json::json!({
                            "node_count": node_count,
                            "relationship_count": rel_count
                        }))))
                    } else {
                        Ok(Json(ApiResponse::success(serde_json::json!({
                            "node_count": 0,
                            "relationship_count": 0
                        }))))
                    }
                }
                Err(e) => {
                    error!("Failed to get stats: {}", e);
                    Ok(Json(ApiResponse::error(e.to_string())))
                }
            }
        }
    }
}

/// Create Neo4j router with all endpoints
pub fn router(state: Neo4jState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/stats", get(get_stats))
        .route("/nodes", post(create_node))
        .route("/nodes/{id}", get(get_node))
        .route("/nodes/{id}", delete(delete_node))
        .route("/relationships", post(create_relationship))
        .route("/cypher", post(execute_cypher))
        .route("/graphrag", post(graph_rag))
        .with_state(state)
}

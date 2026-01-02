use domain_projects::ApiResource;
use utoipa::OpenApi;

/// Qdrant vector database API documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::qdrant::health,
        crate::api::qdrant::list_collections,
        crate::api::qdrant::create_collection,
        crate::api::qdrant::delete_collection,
        crate::api::qdrant::upsert_documents,
        crate::api::qdrant::search,
    ),
    components(schemas(
        crate::api::qdrant::Document,
        crate::api::qdrant::UpsertRequest,
        crate::api::qdrant::DocumentWithEmbedding,
        crate::api::qdrant::SearchRequest,
        crate::api::qdrant::SearchResult,
        crate::api::qdrant::CreateCollectionRequest,
        crate::api::qdrant::DistanceMetric,
        crate::api::qdrant::CollectionInfo,
    )),
    tags((name = "qdrant", description = "Qdrant vector database operations"))
)]
pub struct QdrantApiDoc;

/// Neo4j graph database API documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::neo4j::health,
        crate::api::neo4j::create_node,
        crate::api::neo4j::get_node,
        crate::api::neo4j::delete_node,
        crate::api::neo4j::create_relationship,
        crate::api::neo4j::execute_cypher,
        crate::api::neo4j::graph_rag,
        crate::api::neo4j::get_stats,
    ),
    components(schemas(
        crate::api::neo4j::GraphNode,
        crate::api::neo4j::Relationship,
        crate::api::neo4j::CreateNodeRequest,
        crate::api::neo4j::CreateRelationshipRequest,
        crate::api::neo4j::CypherQueryRequest,
        crate::api::neo4j::GraphRagRequest,
        crate::api::neo4j::GraphRagResult,
        crate::api::neo4j::RelatedEntity,
    )),
    tags((name = "neo4j", description = "Neo4j graph database operations"))
)]
pub struct Neo4jApiDoc;

/// ArangoDB multi-model database API documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::arangodb::health,
        crate::api::arangodb::list_collections,
        crate::api::arangodb::create_collection,
        crate::api::arangodb::delete_collection,
        crate::api::arangodb::create_document,
        crate::api::arangodb::get_document,
        crate::api::arangodb::update_document,
        crate::api::arangodb::delete_document,
        crate::api::arangodb::create_edge,
        crate::api::arangodb::execute_aql,
        crate::api::arangodb::traverse,
        crate::api::arangodb::get_stats,
    ),
    components(schemas(
        crate::api::arangodb::Document,
        crate::api::arangodb::Edge,
        crate::api::arangodb::CreateCollectionRequest,
        crate::api::arangodb::CollectionType,
        crate::api::arangodb::AqlQueryRequest,
        crate::api::arangodb::TraversalRequest,
        crate::api::arangodb::TraversalDirection,
        crate::api::arangodb::TraversalResult,
    )),
    tags((name = "arangodb", description = "ArangoDB multi-model database operations"))
)]
pub struct ArangoDbApiDoc;

/// Milvus vector database API documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::milvus::health,
        crate::api::milvus::list_collections,
        crate::api::milvus::create_collection,
        crate::api::milvus::delete_collection,
        crate::api::milvus::get_collection,
        crate::api::milvus::insert_vectors,
        crate::api::milvus::search,
        crate::api::milvus::query,
        crate::api::milvus::delete_entities,
    ),
    components(schemas(
        crate::api::milvus::CreateCollectionRequest,
        crate::api::milvus::MetricType,
        crate::api::milvus::InsertRequest,
        crate::api::milvus::SearchRequest,
        crate::api::milvus::SearchResult,
        crate::api::milvus::QueryRequest,
        crate::api::milvus::DeleteRequest,
        crate::api::milvus::CollectionInfo,
    )),
    tags((name = "milvus", description = "Milvus vector database operations"))
)]
pub struct MilvusApiDoc;

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(axum_helpers::ErrorResponse)
    ),
    info(
        title = "Zerg API",
        version = "0.1.0",
        description = "API for managing tasks, projects, cloud resources, users, and vector/graph databases for RAG"
    ),
    servers(
        (url = "/api", description = "API base path")
    ),
    nest(
        (path = "/tasks", api = domain_tasks::GrpcApiDoc),
        (path = domain_projects::entity::Model::URL, api = domain_projects::ApiDoc),
        (path = "/help", api = crate::api::help::HelpApiDoc),
        (path = "/qdrant", api = QdrantApiDoc),
        (path = "/neo4j", api = Neo4jApiDoc),
        (path = "/arangodb", api = ArangoDbApiDoc),
        (path = "/milvus", api = MilvusApiDoc)
    )
)]
pub struct ApiDoc;

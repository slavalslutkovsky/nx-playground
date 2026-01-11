mod direct;
mod grpc;

use axum::{
    Router,
    routing::{get, post},
};
use rpc::vector::vector_service_client::VectorServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;
use utoipa::OpenApi;

use crate::repository::VectorRepository;
use crate::service::VectorService;

// Re-export HTTP model types
pub use grpc::{
    CollectionCreateRequest, CollectionResponse, CollectionsListResponse, DeleteResponse,
    SearchRequest, SearchResponse, UpsertRequest, UpsertResponse, VectorGetResponse,
};

/// OpenAPI documentation for Vector API (Direct DB)
#[derive(OpenApi)]
#[openapi(
    paths(
        direct::search_vectors,
        direct::upsert_vector,
        direct::list_collections,
        direct::create_collection,
        direct::get_collection,
        direct::delete_collection,
        direct::get_vectors,
        direct::delete_vectors,
    ),
    components(
        schemas(
            SearchRequest, SearchResponse,
            UpsertRequest, UpsertResponse,
            CollectionCreateRequest, CollectionResponse, CollectionsListResponse,
            VectorGetResponse, DeleteResponse
        )
    ),
    tags(
        (name = "vectors-direct", description = "Direct Qdrant vector operations")
    )
)]
pub struct DirectApiDoc;

/// OpenAPI documentation for Vector API (gRPC)
#[derive(OpenApi)]
#[openapi(
    paths(
        grpc::search_vectors,
        grpc::upsert_vector,
        grpc::list_collections,
        grpc::create_collection,
        grpc::get_collection,
        grpc::delete_collection,
        grpc::get_vectors,
        grpc::delete_vectors,
    ),
    components(
        schemas(
            SearchRequest, SearchResponse,
            UpsertRequest, UpsertResponse,
            CollectionCreateRequest, CollectionResponse, CollectionsListResponse,
            VectorGetResponse, DeleteResponse
        )
    ),
    tags(
        (name = "vectors", description = "gRPC-backed vector operations")
    )
)]
pub struct GrpcApiDoc;

/// Create router for direct Qdrant-backed handlers
pub fn direct_router<R: VectorRepository + 'static>(service: VectorService<R>) -> Router {
    let shared_service = Arc::new(service);

    Router::new()
        .route("/search", post(direct::search_vectors))
        .route("/upsert", post(direct::upsert_vector))
        .route(
            "/collections",
            get(direct::list_collections).post(direct::create_collection),
        )
        .route(
            "/collections/{name}",
            get(direct::get_collection).delete(direct::delete_collection),
        )
        .route(
            "/vectors",
            get(direct::get_vectors).delete(direct::delete_vectors),
        )
        .with_state(shared_service)
}

/// Create router for gRPC-backed handlers
pub fn grpc_router(client: VectorServiceClient<Channel>) -> Router {
    Router::new()
        .route("/search", post(grpc::search_vectors))
        .route("/upsert", post(grpc::upsert_vector))
        .route(
            "/collections",
            get(grpc::list_collections).post(grpc::create_collection),
        )
        .route(
            "/collections/{name}",
            get(grpc::get_collection).delete(grpc::delete_collection),
        )
        .route(
            "/vectors",
            get(grpc::get_vectors).delete(grpc::delete_vectors),
        )
        .with_state(client)
}

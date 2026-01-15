use axum::Router;
use domain_vector::handlers;

/// Creates the router for gRPC-backed vector operations.
/// Routes:
/// - POST /search - Search vectors with automatic embedding
/// - POST /upsert - Upsert vector with automatic embedding
/// - GET /collections - List collections
/// - POST /collections - Create collection
/// - GET /collections/{name} - Get collection info
/// - DELETE /collections/{name} - Delete collection
/// - GET /vectors - Get vectors by IDs
/// - DELETE /vectors - Delete vectors by IDs
pub fn router(state: crate::state::AppState) -> Router {
    handlers::grpc_router(state.vector_client.clone())
}

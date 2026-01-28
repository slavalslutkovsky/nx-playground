use axum::Router;
use domain_vector::handlers;

/// Creates the router for direct Qdrant-backed vector operations.
///
/// Note: This requires a pre-initialized VectorService with QdrantRepository.
/// For now, we delegate to the gRPC-backed routes since direct Qdrant access
/// requires async initialization that should happen at startup.
///
/// In production, you would:
/// 1. Initialize QdrantRepository in main.rs
/// 2. Create VectorService with embedding provider
/// 3. Pass the service here
///
/// Routes mirror the gRPC-backed routes:
/// - POST /search - Search vectors with automatic embedding
/// - POST /upsert - Upsert vector with automatic embedding
/// - GET /collections - List collections
/// - POST /collections - Create collection
/// - GET /collections/{name} - Get collection info
/// - DELETE /collections/{name} - Delete collection
/// - GET /vectors - Get vectors by IDs
/// - DELETE /vectors - Delete vectors by IDs
pub fn router(state: crate::state::AppState) -> Router {
    // For direct routes, we need QdrantRepository which requires async init.
    // Currently using gRPC client as fallback.
    // TODO: Add QdrantRepository to AppState for true direct access.
    handlers::grpc_router(state.vector_client.clone())
}

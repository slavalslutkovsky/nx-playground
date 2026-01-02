//! Help endpoint handlers for API documentation
//!
//! Integrates api-forge with Zerg API to provide comprehensive
//! API documentation and discovery endpoints.

use api_forge::handlers::{HelpState, help_router};
use axum::Router;
use tokio::sync::OnceCell;
use utoipa::OpenApi;

/// Cached OpenAPI JSON to avoid regenerating on every request
static OPENAPI_JSON: OnceCell<String> = OnceCell::const_new();

/// Create the help router with the OpenAPI spec from the main API
pub fn router() -> Router {
    // Create the help state with API metadata
    let state = HelpState::new("Zerg API", "0.1.0");

    // Initialize the registry from the OpenAPI spec asynchronously
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = initialize_registry(&state_clone).await {
            tracing::error!("Failed to initialize API registry: {}", e);
        }
    });

    help_router(state)
}

/// Initialize the API registry from the OpenAPI spec
async fn initialize_registry(state: &HelpState) -> api_forge::Result<()> {
    // Get or generate the OpenAPI JSON
    let json = OPENAPI_JSON
        .get_or_init(|| async {
            let doc = crate::openapi::ApiDoc::openapi();
            doc.to_json().unwrap_or_else(|_| "{}".to_string())
        })
        .await;

    state.init_from_openapi_json(json).await?;
    tracing::info!(
        "API registry initialized with {} bytes of OpenAPI spec",
        json.len()
    );
    Ok(())
}

/// OpenAPI documentation for the help module (re-export from api-forge)
pub use api_forge::handlers::help::HelpApiDoc;

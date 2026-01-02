//! OpenAPI documentation configuration

use utoipa::OpenApi;

/// Combined OpenAPI documentation for all APIs
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Zerg MongoDB API",
        version = "0.1.0",
        description = "MongoDB-based REST API for managing items and events",
        license(name = "MIT")
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server")
    ),
    nest(
        (path = "/api/items", api = domain_items::ApiDoc),
        (path = "/api/events", api = domain_events::ApiDoc)
    ),
    tags(
        (name = "Items", description = "Item management endpoints (MongoDB)"),
        (name = "events", description = "Event management with MongoDB, InfluxDB, and Dapr")
    )
)]
pub struct ApiDoc;

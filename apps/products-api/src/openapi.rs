//! OpenAPI documentation configuration

use utoipa::OpenApi;

/// Combined OpenAPI documentation for Products API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Products API",
        version = "0.1.0",
        description = "Product/Inventory management API with gRPC and REST support",
        license(name = "MIT")
    ),
    servers(
        (url = "http://localhost:3003", description = "Local development server")
    ),
    nest(
        (path = "/api/products", api = domain_products::ApiDoc)
    ),
    tags(
        (name = "Products", description = "Product management endpoints")
    )
)]
pub struct ApiDoc;

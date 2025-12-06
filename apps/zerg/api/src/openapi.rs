use domain_projects::ApiResource;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(axum_helpers::ErrorResponse)
    ),
    info(
        title = "Zerg API",
        version = "0.1.0",
        description = "API for managing tasks, projects, cloud resources, and users"
    ),
    servers(
        (url = "/api", description = "API base path")
    ),
    nest(
        (path = "/tasks", api = domain_tasks::GrpcApiDoc),
        (path = domain_projects::entity::Model::URL, api = domain_projects::ApiDoc)
    )
)]
pub struct ApiDoc;

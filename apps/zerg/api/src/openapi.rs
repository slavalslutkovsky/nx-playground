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
        (path = "/tasks-direct", api = domain_tasks::DirectApiDoc),
        (path = domain_projects::entity::Model::URL, api = domain_projects::ApiDoc),
        (path = "/users", api = domain_users::ApiDoc),
        (path = "/auth", api = domain_users::AuthApiDoc),
        (path = "/cloud-resources", api = domain_cloud_resources::ApiDoc),
        (path = "/vector", api = domain_vector::VectorApiDoc)
    )
)]
pub struct ApiDoc;

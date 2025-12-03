use domain_projects::ApiResource;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::tasks::list_tasks,
        crate::api::tasks::get_task,
        crate::api::tasks::create_task,
        crate::api::tasks::delete_task,
    ),
    components(
        schemas(
            crate::api::tasks::TaskDto,
            crate::api::tasks::CreateTaskDto,
            axum_helpers::ErrorResponse
        )
    ),
    tags(
        (name = "tasks", description = "Task management endpoints")
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
        (path = domain_projects::entity::Model::API_URL, api = domain_projects::ApiDoc)
    )
)]
pub struct ApiDoc;

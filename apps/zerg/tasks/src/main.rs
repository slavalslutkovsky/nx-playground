use std::pin::Pin;
use std::sync::Arc;

use core_config::FromEnv;
use database::postgres::PostgresConfig;
use domain_tasks::{CreateTask, PgTaskRepository, TaskFilter, TaskService, UpdateTask};
use rpc::tasks::{
    tasks_service_server::{TasksService, TasksServiceServer},
    CreateRequest, CreateResponse, DeleteByIdRequest, DeleteByIdResponse, GetByIdRequest,
    GetByIdResponse, ListRequest, ListResponse, ListStreamRequest, ListStreamResponse,
    UpdateByIdRequest, UpdateByIdResponse,
};
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod conversions;
use conversions::*;

type TaskStream = Pin<Box<dyn Stream<Item = Result<ListStreamResponse, Status>> + Send>>;

pub struct TasksServiceImpl<R>
where
    R: domain_tasks::TaskRepository + 'static,
{
    service: Arc<TaskService<R>>,
}

impl<R> TasksServiceImpl<R>
where
    R: domain_tasks::TaskRepository + 'static,
{
    pub fn new(service: TaskService<R>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}


#[tonic::async_trait]
impl<R> TasksService for TasksServiceImpl<R>
where
    R: domain_tasks::TaskRepository + 'static,
{
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        let priority = proto_priority_to_domain(req.priority)?;
        let status = proto_status_to_domain(req.status)?;
        let project_id = opt_bytes_to_uuid(req.project_id)?;
        let due_date = opt_timestamp_to_datetime(req.due_date);

        let input = CreateTask {
            title: req.title,
            description: req.description,
            project_id,
            priority,
            status,
            due_date,
        };

        let task = self
            .service
            .create_task(input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateResponse {
            id: uuid_to_bytes(task.id),
            title: task.title,
            description: task.description,
            completed: task.completed,
            project_id: opt_uuid_to_bytes(task.project_id),
            priority: domain_priority_to_proto(&task.priority),
            status: domain_status_to_proto(&task.status),
            due_date: opt_datetime_to_timestamp(task.due_date),
            created_at: datetime_to_timestamp(task.created_at),
            updated_at: datetime_to_timestamp(task.updated_at),
        }))
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        let req = request.into_inner();
        let id = bytes_to_uuid(&req.id)?;

        let task = self
            .service
            .get_task(id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetByIdResponse {
            id: uuid_to_bytes(task.id),
            title: task.title,
            description: task.description,
            completed: task.completed,
            project_id: opt_uuid_to_bytes(task.project_id),
            priority: domain_priority_to_proto(&task.priority),
            status: domain_status_to_proto(&task.status),
            due_date: opt_datetime_to_timestamp(task.due_date),
            created_at: datetime_to_timestamp(task.created_at),
            updated_at: datetime_to_timestamp(task.updated_at),
        }))
    }

    async fn delete_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<DeleteByIdResponse>, Status> {
        let req = request.into_inner();
        let id = bytes_to_uuid(&req.id)?;

        self.service
            .delete_task(id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("Deleted task: {}", id);
        Ok(Response::new(DeleteByIdResponse {}))
    }

    async fn update_by_id(
        &self,
        request: Request<UpdateByIdRequest>,
    ) -> Result<Response<UpdateByIdResponse>, Status> {
        let req = request.into_inner();
        let id = bytes_to_uuid(&req.id)?;

        let priority = opt_proto_priority_to_domain(req.priority)?;
        let status = opt_proto_status_to_domain(req.status)?;

        // Convert Option<Vec<u8>> to Option<Option<Uuid>>
        let project_id = req.project_id
            .map(|bytes| bytes_to_uuid(&bytes).map(Some))
            .transpose()?;

        // Convert Option<i64> to Option<Option<DateTime>>
        let due_date = req.due_date
            .map(|ts| Some(timestamp_to_datetime(ts)));

        let input = UpdateTask {
            title: req.title,
            description: req.description,
            completed: req.completed,
            project_id,
            priority,
            status,
            due_date,
        };

        let task = self
            .service
            .update_task(id, input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateByIdResponse {
            id: uuid_to_bytes(task.id),
            title: task.title,
            description: task.description,
            completed: task.completed,
            project_id: opt_uuid_to_bytes(task.project_id),
            priority: domain_priority_to_proto(&task.priority),
            status: domain_status_to_proto(&task.status),
            due_date: opt_datetime_to_timestamp(task.due_date),
            created_at: datetime_to_timestamp(task.created_at),
            updated_at: datetime_to_timestamp(task.updated_at),
        }))
    }

    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let req = request.into_inner();

        let project_id = opt_bytes_to_uuid(req.project_id)?;
        let status = opt_proto_status_to_domain(req.status)?;
        let priority = opt_proto_priority_to_domain(req.priority)?;

        let limit = req.limit as usize;
        let offset = req.offset as usize;

        let filter = TaskFilter {
            project_id,
            status,
            priority,
            completed: req.completed,
            limit,
            offset,
        };

        let tasks = self
            .service
            .list_tasks(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let data: Vec<CreateResponse> = tasks
            .into_iter()
            .map(|task| CreateResponse {
                id: uuid_to_bytes(task.id),
                title: task.title,
                description: task.description,
                completed: task.completed,
                project_id: opt_uuid_to_bytes(task.project_id),
                priority: domain_priority_to_proto(&task.priority),
                status: domain_status_to_proto(&task.status),
                due_date: opt_datetime_to_timestamp(task.due_date),
                created_at: datetime_to_timestamp(task.created_at),
                updated_at: datetime_to_timestamp(task.updated_at),
            })
            .collect();

        Ok(Response::new(ListResponse { data }))
    }

    type ListStreamStream = TaskStream;

    async fn list_stream(
        &self,
        request: Request<ListStreamRequest>,
    ) -> Result<Response<Self::ListStreamStream>, Status> {
        let req = request.into_inner();

        let project_id = opt_bytes_to_uuid(req.project_id)?;
        let status = opt_proto_status_to_domain(req.status)?;
        let priority = opt_proto_priority_to_domain(req.priority)?;

        let limit = req.limit as usize;

        let filter = TaskFilter {
            project_id,
            status,
            priority,
            completed: req.completed,
            limit,
            offset: 0,
        };

        let tasks = self
            .service
            .list_tasks(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = tokio_stream::iter(tasks.into_iter().map(|task| {
            Ok(ListStreamResponse {
                id: uuid_to_bytes(task.id),
                title: task.title,
                description: task.description,
                completed: task.completed,
                project_id: opt_uuid_to_bytes(task.project_id),
                priority: domain_priority_to_proto(&task.priority),
                status: domain_status_to_proto(&task.status),
                due_date: opt_datetime_to_timestamp(task.due_date),
                created_at: datetime_to_timestamp(task.created_at),
                updated_at: datetime_to_timestamp(task.updated_at),
            })
        }));

        Ok(Response::new(Box::pin(stream)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load config
    let config = PostgresConfig::from_env()?;

    // Connect to database
    info!("Connecting to database...");
    let db = database::postgres::connect_from_config_with_retry(config, None).await?;
    info!("Connected to database");

    // Create repository and service
    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);

    // Create gRPC service
    let tasks_service = TasksServiceImpl::new(service);

    let addr = "[::1]:50051".parse()?;
    info!("TasksService listening on {}", addr);

    Server::builder()
        .add_service(
            TasksServiceServer::new(tasks_service)
                // Enable zstd compression for requests and responses (3-5x faster than gzip)
                .accept_compressed(tonic::codec::CompressionEncoding::Zstd)
                .send_compressed(tonic::codec::CompressionEncoding::Zstd)
        )
        .serve(addr)
        .await?;

    Ok(())
}

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use rpc::tasks::{
    tasks_service_server::{TasksService, TasksServiceServer},
    CreateRequest, CreateResponse, DeleteByIdRequest, DeleteByIdResponse, GetByIdRequest,
    GetByIdResponse, ListRequest, ListResponse, ListStreamRequest, ListStreamResponse,
    UpdateByIdRequest, UpdateByIdResponse,
};
use tokio::sync::RwLock;
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

type TaskStream = Pin<Box<dyn Stream<Item = Result<ListStreamResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct TasksServiceImpl {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

#[derive(Debug, Clone)]
struct Task {
    id: String,
    title: String,
    description: String,
    completed: String,
}

#[tonic::async_trait]
impl TasksService for TasksServiceImpl {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::new_v4().to_string();

        let task = Task {
            id: id.clone(),
            title: req.title.clone(),
            description: req.description.clone(),
            completed: req.completed.clone(),
        };

        self.tasks.write().await.insert(id.clone(), task);

        info!("Created task: {}", id);

        Ok(Response::new(CreateResponse {
            id,
            title: req.title,
            description: req.description,
            completed: req.completed,
        }))
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        let req = request.into_inner();
        let tasks = self.tasks.read().await;

        match tasks.get(&req.id) {
            Some(task) => Ok(Response::new(GetByIdResponse {
                id: task.id.clone(),
                title: task.title.clone(),
                description: task.description.clone(),
                completed: task.completed.clone(),
            })),
            None => Err(Status::not_found(format!("Task {} not found", req.id))),
        }
    }

    async fn delete_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<DeleteByIdResponse>, Status> {
        let req = request.into_inner();
        let mut tasks = self.tasks.write().await;

        match tasks.remove(&req.id) {
            Some(_) => {
                info!("Deleted task: {}", req.id);
                Ok(Response::new(DeleteByIdResponse {}))
            }
            None => Err(Status::not_found(format!("Task {} not found", req.id))),
        }
    }

    async fn update_by_id(
        &self,
        request: Request<UpdateByIdRequest>,
    ) -> Result<Response<UpdateByIdResponse>, Status> {
        let req = request.into_inner();
        let tasks = self.tasks.read().await;

        match tasks.get(&req.id) {
            Some(task) => Ok(Response::new(UpdateByIdResponse {
                id: task.id.clone(),
                title: task.title.clone(),
                description: task.description.clone(),
                completed: task.completed.clone(),
            })),
            None => Err(Status::not_found(format!("Task {} not found", req.id))),
        }
    }

    async fn list(&self, _request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let tasks = self.tasks.read().await;

        let data: Vec<CreateResponse> = tasks
            .values()
            .map(|task| CreateResponse {
                id: task.id.clone(),
                title: task.title.clone(),
                description: task.description.clone(),
                completed: task.completed.clone(),
            })
            .collect();

        Ok(Response::new(ListResponse { data }))
    }

    type ListStreamStream = TaskStream;

    async fn list_stream(
        &self,
        _request: Request<ListStreamRequest>,
    ) -> Result<Response<Self::ListStreamStream>, Status> {
        let tasks = self.tasks.read().await;
        let tasks_vec: Vec<Task> = tasks.values().cloned().collect();

        let stream = tokio_stream::iter(tasks_vec.into_iter().map(|task| {
            Ok(ListStreamResponse {
                id: task.id,
                title: task.title,
                description: task.description,
                completed: task.completed,
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

    let addr = "[::1]:50051".parse()?;
    let tasks_service = TasksServiceImpl::default();

    info!("TasksService listening on {}", addr);

    Server::builder()
        .add_service(TasksServiceServer::new(tasks_service))
        .serve(addr)
        .await?;

    Ok(())
}

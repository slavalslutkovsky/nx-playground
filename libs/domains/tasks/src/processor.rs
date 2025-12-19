//! Task processor for stream workers.
//!
//! This module provides the `TaskProcessor` that implements `StreamProcessor<TaskCommand>`,
//! handling CRUD operations on tasks via Redis streams.

use crate::models::TaskFilter;
use crate::repository::TaskRepository;
use crate::service::TaskService;
use crate::stream_models::{
    TaskCommand, TaskCommandPayload, TaskCommandResult, TaskResultData,
};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use stream_worker::{StreamError, StreamProcessor, StreamProducer};
use tracing::{error, info};

/// Task processor that handles CRUD operations.
///
/// This processor handles `TaskCommand` items from the stream,
/// executes the appropriate service method, and sends results
/// to the results stream.
pub struct TaskProcessor<R: TaskRepository> {
    service: Arc<TaskService<R>>,
    result_producer: Option<StreamProducer>,
}

impl<R: TaskRepository + 'static> TaskProcessor<R> {
    /// Create a new task processor.
    pub fn new(service: TaskService<R>) -> Self {
        Self {
            service: Arc::new(service),
            result_producer: None,
        }
    }

    /// Create a processor with result producer for async responses.
    pub fn with_result_producer(service: TaskService<R>, redis: ConnectionManager) -> Self {
        use crate::streams::TaskResultStream;

        Self {
            service: Arc::new(service),
            result_producer: Some(StreamProducer::from_stream_def::<TaskResultStream>(redis)),
        }
    }

    /// Send a result to the results stream (if configured).
    async fn send_result(&self, result: &TaskCommandResult) -> Result<(), StreamError> {
        if let Some(producer) = &self.result_producer {
            producer.send(result).await?;
        }
        Ok(())
    }

    /// Process a Create command.
    async fn process_create(
        &self,
        cmd: &TaskCommand,
        input: &crate::models::CreateTask,
    ) -> TaskCommandResult {
        match self.service.create_task(input.clone()).await {
            Ok(task) => {
                info!(task_id = %task.id, "Created task via stream");
                TaskCommandResult::success(
                    &cmd.correlation_id,
                    TaskResultData::Single { task },
                )
            }
            Err(e) => {
                error!(error = %e, "Failed to create task");
                TaskCommandResult::failure(&cmd.correlation_id, e.to_string())
            }
        }
    }

    /// Process a GetById command.
    async fn process_get_by_id(&self, cmd: &TaskCommand, task_id: uuid::Uuid) -> TaskCommandResult {
        match self.service.get_task(task_id).await {
            Ok(task) => TaskCommandResult::success(
                &cmd.correlation_id,
                TaskResultData::Single { task },
            ),
            Err(e) => TaskCommandResult::failure(&cmd.correlation_id, e.to_string()),
        }
    }

    /// Process an Update command.
    async fn process_update(
        &self,
        cmd: &TaskCommand,
        task_id: uuid::Uuid,
        data: &crate::models::UpdateTask,
    ) -> TaskCommandResult {
        match self.service.update_task(task_id, data.clone()).await {
            Ok(task) => {
                info!(task_id = %task.id, "Updated task via stream");
                TaskCommandResult::success(
                    &cmd.correlation_id,
                    TaskResultData::Single { task },
                )
            }
            Err(e) => {
                error!(error = %e, task_id = %task_id, "Failed to update task");
                TaskCommandResult::failure(&cmd.correlation_id, e.to_string())
            }
        }
    }

    /// Process a Delete command.
    async fn process_delete(&self, cmd: &TaskCommand, task_id: uuid::Uuid) -> TaskCommandResult {
        match self.service.delete_task(task_id).await {
            Ok(()) => {
                info!(task_id = %task_id, "Deleted task via stream");
                TaskCommandResult::success(
                    &cmd.correlation_id,
                    TaskResultData::Deleted { id: task_id },
                )
            }
            Err(e) => {
                error!(error = %e, task_id = %task_id, "Failed to delete task");
                TaskCommandResult::failure(&cmd.correlation_id, e.to_string())
            }
        }
    }

    /// Process a List command.
    async fn process_list(&self, cmd: &TaskCommand, filter: &TaskFilter) -> TaskCommandResult {
        match self.service.list_tasks(filter.clone()).await {
            Ok(tasks) => TaskCommandResult::success(
                &cmd.correlation_id,
                TaskResultData::List { tasks },
            ),
            Err(e) => TaskCommandResult::failure(&cmd.correlation_id, e.to_string()),
        }
    }

    /// Process a Complete command.
    async fn process_complete(&self, cmd: &TaskCommand, task_id: uuid::Uuid) -> TaskCommandResult {
        match self.service.complete_task(task_id).await {
            Ok(task) => {
                info!(task_id = %task.id, "Completed task via stream");
                TaskCommandResult::success(
                    &cmd.correlation_id,
                    TaskResultData::Single { task },
                )
            }
            Err(e) => TaskCommandResult::failure(&cmd.correlation_id, e.to_string()),
        }
    }

    /// Process an Uncomplete command.
    async fn process_uncomplete(
        &self,
        cmd: &TaskCommand,
        task_id: uuid::Uuid,
    ) -> TaskCommandResult {
        match self.service.uncomplete_task(task_id).await {
            Ok(task) => {
                info!(task_id = %task.id, "Uncompleted task via stream");
                TaskCommandResult::success(
                    &cmd.correlation_id,
                    TaskResultData::Single { task },
                )
            }
            Err(e) => TaskCommandResult::failure(&cmd.correlation_id, e.to_string()),
        }
    }
}

#[async_trait]
impl<R: TaskRepository + 'static> StreamProcessor<TaskCommand> for TaskProcessor<R> {
    async fn process(&self, cmd: &TaskCommand) -> Result<(), StreamError> {
        info!(
            command_id = %cmd.id,
            action = %cmd.action,
            correlation_id = %cmd.correlation_id,
            retry_count = %cmd.retry_count,
            "Processing task command"
        );

        let result = match &cmd.payload {
            TaskCommandPayload::Create(input) => self.process_create(cmd, input).await,
            TaskCommandPayload::GetById { task_id } => self.process_get_by_id(cmd, *task_id).await,
            TaskCommandPayload::Update { task_id, data } => {
                self.process_update(cmd, *task_id, data).await
            }
            TaskCommandPayload::Delete { task_id } => self.process_delete(cmd, *task_id).await,
            TaskCommandPayload::List(filter) => self.process_list(cmd, filter).await,
            TaskCommandPayload::Complete { task_id } => self.process_complete(cmd, *task_id).await,
            TaskCommandPayload::Uncomplete { task_id } => {
                self.process_uncomplete(cmd, *task_id).await
            }
        };

        // Send result to results stream
        self.send_result(&result).await?;

        // If the command failed, propagate as error for retry logic
        if !result.success {
            return Err(StreamError::Processing(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "TaskProcessor"
    }

    async fn health_check(&self) -> Result<bool, StreamError> {
        // Could add database health check here
        Ok(true)
    }
}

impl<R: TaskRepository> Clone for TaskProcessor<R> {
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
            result_producer: self.result_producer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_name() {
        assert_eq!("TaskProcessor", "TaskProcessor");
    }
}

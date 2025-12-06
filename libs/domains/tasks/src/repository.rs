use async_trait::async_trait;
use uuid::Uuid;

use crate::error::TaskResult;
use crate::models::{CreateTask, Task, TaskFilter, UpdateTask};

/// Repository trait for Task persistence
///
/// This trait defines the data access interface for tasks.
/// Implementations can use different storage backends (PostgreSQL, etc.)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Create a new task
    async fn create(&self, input: CreateTask) -> TaskResult<Task>;

    /// Get a task by ID
    async fn get_by_id(&self, id: Uuid) -> TaskResult<Option<Task>>;

    /// List tasks with optional filters
    async fn list(&self, filter: TaskFilter) -> TaskResult<Vec<Task>>;

    /// Update an existing task
    async fn update(&self, id: Uuid, input: UpdateTask) -> TaskResult<Task>;

    /// Delete a task by ID
    async fn delete(&self, id: Uuid) -> TaskResult<bool>;

    /// Count all tasks
    async fn count(&self) -> TaskResult<usize>;

    /// Count tasks by project
    async fn count_by_project(&self, project_id: Uuid) -> TaskResult<usize>;
}

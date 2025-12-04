use async_trait::async_trait;
use uuid::Uuid;

use crate::error::ProjectResult;
use crate::models::{CreateProject, Project, ProjectFilter, UpdateProject};

/// Repository trait for Project persistence
///
/// This trait defines the data access interface for projects.
/// Implementations can use different storage backends (PostgreSQL, etc.)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Create a new project
    async fn create(&self, input: CreateProject) -> ProjectResult<Project>;

    /// Get a project by ID
    async fn get_by_id(&self, id: Uuid) -> ProjectResult<Option<Project>>;

    /// List projects with optional filters
    async fn list(&self, filter: ProjectFilter) -> ProjectResult<Vec<Project>>;

    /// Update an existing project
    async fn update(&self, id: Uuid, input: UpdateProject) -> ProjectResult<Project>;

    /// Delete a project by ID
    async fn delete(&self, id: Uuid) -> ProjectResult<bool>;

    /// Check if a project name exists for a user
    async fn exists_by_name(&self, user_id: Uuid, name: &str) -> ProjectResult<bool>;

    /// Count projects for a user
    async fn count_by_user(&self, user_id: Uuid) -> ProjectResult<usize>;
}

use std::sync::Arc;
use uuid::Uuid;

use crate::error::{ProjectError, ProjectResult};
use crate::models::{CreateProject, Project, ProjectFilter, ProjectStatus, UpdateProject};
use crate::repository::ProjectRepository;

/// Service layer for Project business logic
#[derive(Clone)]
pub struct ProjectService<R: ProjectRepository> {
    repository: Arc<R>,
}

impl<R: ProjectRepository> ProjectService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Create a new project with validation
    pub async fn create_project(&self, input: CreateProject) -> ProjectResult<Project> {
        // Validate input
        self.validate_create(&input)?;

        self.repository.create(input).await
    }

    /// Get a project by ID
    pub async fn get_project(&self, id: Uuid) -> ProjectResult<Project> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or(ProjectError::NotFound(id))
    }

    /// Get a project by ID, verifying user ownership
    pub async fn get_project_for_user(&self, id: Uuid, user_id: Uuid) -> ProjectResult<Project> {
        let project = self.get_project(id).await?;

        if project.user_id != user_id {
            return Err(ProjectError::Unauthorized(id));
        }

        Ok(project)
    }

    /// List projects with filters
    pub async fn list_projects(&self, filter: ProjectFilter) -> ProjectResult<Vec<Project>> {
        self.repository.list(filter).await
    }

    /// Update a project
    pub async fn update_project(&self, id: Uuid, input: UpdateProject) -> ProjectResult<Project> {
        // Validate input
        self.validate_update(&input)?;

        self.repository.update(id, input).await
    }

    /// Update a project, verifying user ownership
    pub async fn update_project_for_user(
        &self,
        id: Uuid,
        user_id: Uuid,
        input: UpdateProject,
    ) -> ProjectResult<Project> {
        let project = self.get_project(id).await?;

        if project.user_id != user_id {
            return Err(ProjectError::Unauthorized(id));
        }

        self.update_project(id, input).await
    }

    /// Delete a project
    pub async fn delete_project(&self, id: Uuid) -> ProjectResult<()> {
        let deleted = self.repository.delete(id).await?;

        if !deleted {
            return Err(ProjectError::NotFound(id));
        }

        Ok(())
    }

    /// Delete a project, verifying user ownership
    pub async fn delete_project_for_user(&self, id: Uuid, user_id: Uuid) -> ProjectResult<()> {
        let project = self.get_project(id).await?;

        if project.user_id != user_id {
            return Err(ProjectError::Unauthorized(id));
        }

        self.delete_project(id).await
    }

    /// Activate a project (change status to Active)
    pub async fn activate_project(&self, id: Uuid) -> ProjectResult<Project> {
        let project = self.get_project(id).await?;

        if project.status == ProjectStatus::Active {
            return Ok(project);
        }

        if project.status == ProjectStatus::Deleting {
            return Err(ProjectError::Validation(
                "Cannot activate a project being deleted".to_string(),
            ));
        }

        self.repository
            .update(
                id,
                UpdateProject {
                    status: Some(ProjectStatus::Active),
                    ..Default::default()
                },
            )
            .await
    }

    /// Suspend a project
    pub async fn suspend_project(&self, id: Uuid) -> ProjectResult<Project> {
        let project = self.get_project(id).await?;

        if project.status == ProjectStatus::Suspended {
            return Ok(project);
        }

        if project.status != ProjectStatus::Active {
            return Err(ProjectError::Validation(
                "Only active projects can be suspended".to_string(),
            ));
        }

        self.repository
            .update(
                id,
                UpdateProject {
                    status: Some(ProjectStatus::Suspended),
                    ..Default::default()
                },
            )
            .await
    }

    /// Archive a project
    pub async fn archive_project(&self, id: Uuid) -> ProjectResult<Project> {
        self.repository
            .update(
                id,
                UpdateProject {
                    status: Some(ProjectStatus::Archived),
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .await
    }

    // Validation helpers

    fn validate_create(&self, input: &CreateProject) -> ProjectResult<()> {
        if input.name.trim().is_empty() {
            return Err(ProjectError::Validation(
                "Project name cannot be empty".to_string(),
            ));
        }

        if input.name.len() > 100 {
            return Err(ProjectError::Validation(
                "Project name cannot exceed 100 characters".to_string(),
            ));
        }

        if !input
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ProjectError::Validation(
                "Project name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        if input.region.trim().is_empty() {
            return Err(ProjectError::Validation(
                "Region cannot be empty".to_string(),
            ));
        }

        if let Some(budget) = input.budget_limit {
            if budget < 0.0 {
                return Err(ProjectError::Validation(
                    "Budget limit cannot be negative".to_string(),
                ));
            }
        }

        // Validate tags
        for tag in &input.tags {
            if tag.key.trim().is_empty() {
                return Err(ProjectError::Validation(
                    "Tag key cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_update(&self, input: &UpdateProject) -> ProjectResult<()> {
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(ProjectError::Validation(
                    "Project name cannot be empty".to_string(),
                ));
            }

            if name.len() > 100 {
                return Err(ProjectError::Validation(
                    "Project name cannot exceed 100 characters".to_string(),
                ));
            }

            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(ProjectError::Validation(
                    "Project name can only contain alphanumeric characters, hyphens, and underscores"
                        .to_string(),
                ));
            }
        }

        if let Some(budget) = input.budget_limit {
            if budget < 0.0 {
                return Err(ProjectError::Validation(
                    "Budget limit cannot be negative".to_string(),
                ));
            }
        }

        if let Some(ref tags) = input.tags {
            for tag in tags {
                if tag.key.trim().is_empty() {
                    return Err(ProjectError::Validation(
                        "Tag key cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Default for UpdateProject {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            region: None,
            environment: None,
            status: None,
            budget_limit: None,
            tags: None,
            enabled: None,
        }
    }
}

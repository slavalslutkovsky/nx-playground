use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{ProjectError, ProjectResult};
use crate::models::{CreateProject, Project, ProjectFilter, UpdateProject};

/// Repository trait for Project persistence
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
}

/// In-memory implementation of ProjectRepository (for development/testing)
#[derive(Debug, Default, Clone)]
pub struct InMemoryProjectRepository {
    projects: Arc<RwLock<HashMap<Uuid, Project>>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn create(&self, input: CreateProject) -> ProjectResult<Project> {
        let mut projects = self.projects.write().await;

        // Check for duplicate name
        let name_exists = projects.values().any(|p| {
            p.user_id == input.user_id && p.name.to_lowercase() == input.name.to_lowercase()
        });

        if name_exists {
            return Err(ProjectError::DuplicateName(input.name));
        }

        let project = Project::new(input);
        projects.insert(project.id, project.clone());

        tracing::info!(project_id = %project.id, "Created project");
        Ok(project)
    }

    async fn get_by_id(&self, id: Uuid) -> ProjectResult<Option<Project>> {
        let projects = self.projects.read().await;
        Ok(projects.get(&id).cloned())
    }

    async fn list(&self, filter: ProjectFilter) -> ProjectResult<Vec<Project>> {
        let projects = self.projects.read().await;

        let mut result: Vec<Project> = projects
            .values()
            .filter(|p| {
                if let Some(user_id) = filter.user_id {
                    if p.user_id != user_id {
                        return false;
                    }
                }
                if let Some(provider) = filter.cloud_provider {
                    if p.cloud_provider != provider {
                        return false;
                    }
                }
                if let Some(env) = filter.environment {
                    if p.environment != env {
                        return false;
                    }
                }
                if let Some(status) = filter.status {
                    if p.status != status {
                        return false;
                    }
                }
                if let Some(enabled) = filter.enabled {
                    if p.enabled != enabled {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by created_at descending (newest first)
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let result: Vec<Project> = result
            .into_iter()
            .skip(filter.offset)
            .take(filter.limit)
            .collect();

        Ok(result)
    }

    async fn update(&self, id: Uuid, input: UpdateProject) -> ProjectResult<Project> {
        let mut projects = self.projects.write().await;

        // First, get the user_id for duplicate check
        let user_id = projects
            .get(&id)
            .ok_or(ProjectError::NotFound(id))?
            .user_id;

        // Check for duplicate name if name is being changed
        if let Some(ref new_name) = input.name {
            let name_exists = projects.values().any(|p| {
                p.id != id
                    && p.user_id == user_id
                    && p.name.to_lowercase() == new_name.to_lowercase()
            });

            if name_exists {
                return Err(ProjectError::DuplicateName(new_name.clone()));
            }
        }

        // Now get mutable reference and apply update
        let project = projects.get_mut(&id).unwrap();
        project.apply_update(input);
        let updated = project.clone();

        tracing::info!(project_id = %id, "Updated project");
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> ProjectResult<bool> {
        let mut projects = self.projects.write().await;

        if projects.remove(&id).is_some() {
            tracing::info!(project_id = %id, "Deleted project");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists_by_name(&self, user_id: Uuid, name: &str) -> ProjectResult<bool> {
        let projects = self.projects.read().await;
        let exists = projects
            .values()
            .any(|p| p.user_id == user_id && p.name.to_lowercase() == name.to_lowercase());
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CloudProvider;

    #[tokio::test]
    async fn test_create_and_get_project() {
        let repo = InMemoryProjectRepository::new();

        let input = CreateProject {
            name: "test-project".to_string(),
            user_id: Uuid::new_v4(),
            description: "A test project".to_string(),
            cloud_provider: CloudProvider::Aws,
            region: "us-east-1".to_string(),
            environment: Default::default(),
            budget_limit: Some(100.0),
            tags: vec![],
        };

        let project = repo.create(input).await.unwrap();
        assert_eq!(project.name, "test-project");

        let fetched = repo.get_by_id(project.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, project.id);
    }

    #[tokio::test]
    async fn test_duplicate_name_error() {
        let repo = InMemoryProjectRepository::new();
        let user_id = Uuid::new_v4();

        let input = CreateProject {
            name: "my-project".to_string(),
            user_id,
            description: String::new(),
            cloud_provider: CloudProvider::Gcp,
            region: "europe-west1".to_string(),
            environment: Default::default(),
            budget_limit: None,
            tags: vec![],
        };

        repo.create(input.clone()).await.unwrap();

        let result = repo.create(input).await;
        assert!(matches!(result, Err(ProjectError::DuplicateName(_))));
    }
}

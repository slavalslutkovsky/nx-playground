use async_trait::async_trait;
use database::BaseRepository;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

use crate::{
    entity,
    error::{ProjectError, ProjectResult},
    models::{CreateProject, Project, ProjectFilter, UpdateProject},
    repository::ProjectRepository,
};

pub struct PgProjectRepository {
    base: BaseRepository<entity::Entity>,
}

impl PgProjectRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            base: BaseRepository::new(db),
        }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, input: CreateProject) -> ProjectResult<Project> {
        // Check for duplicate name
        let exists = self.exists_by_name(input.user_id, &input.name).await?;
        if exists {
            return Err(ProjectError::DuplicateName(input.name));
        }

        // Convert CreateProject to ActiveModel
        let active_model: entity::ActiveModel = input.into();

        // Insert using base repository
        let model = self
            .base
            .insert(active_model)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        tracing::info!(project_id = %model.id, "Created project");
        Ok(model.into())
    }

    async fn get_by_id(&self, id: Uuid) -> ProjectResult<Option<Project>> {
        let model = self
            .base
            .find_by_id(id)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        Ok(model.map(|m| m.into()))
    }

    async fn list(&self, filter: ProjectFilter) -> ProjectResult<Vec<Project>> {
        let mut query = entity::Entity::find();

        // Apply filters
        if let Some(user_id) = filter.user_id {
            query = query.filter(entity::Column::UserId.eq(user_id));
        }

        if let Some(cloud_provider) = filter.cloud_provider {
            query = query.filter(entity::Column::CloudProvider.eq(cloud_provider.to_string()));
        }

        if let Some(environment) = filter.environment {
            query = query.filter(entity::Column::Environment.eq(environment.to_string()));
        }

        if let Some(status) = filter.status {
            query = query.filter(entity::Column::Status.eq(status.to_string()));
        }

        if let Some(enabled) = filter.enabled {
            query = query.filter(entity::Column::Enabled.eq(enabled));
        }

        // Apply pagination and ordering
        query = query
            .order_by_desc(entity::Column::CreatedAt)
            .limit(filter.limit as u64)
            .offset(filter.offset as u64);

        let models = query
            .all(self.base.db())
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    async fn update(&self, id: Uuid, input: UpdateProject) -> ProjectResult<Project> {
        // Fetch existing project
        let model = self
            .base
            .find_by_id(id)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?
            .ok_or(ProjectError::NotFound(id))?;

        // Check for duplicate name if name is being changed
        if let Some(ref new_name) = input.name {
            let name_exists = entity::Entity::find()
                .filter(entity::Column::UserId.eq(model.user_id))
                .filter(entity::Column::Name.eq(new_name))
                .filter(entity::Column::Id.ne(id))
                .one(self.base.db())
                .await
                .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?
                .is_some();

            if name_exists {
                return Err(ProjectError::DuplicateName(new_name.clone()));
            }
        }

        // Convert to domain model
        let mut project: Project = model.into();

        // Apply updates
        project.apply_update(input);

        // Convert back to ActiveModel for update
        let active_model: entity::ActiveModel = entity::ActiveModel {
            id: Set(project.id),
            name: Set(project.name.clone()),
            user_id: Set(project.user_id),
            description: Set(project.description.clone()),
            cloud_provider: Set(project.cloud_provider.to_string()),
            region: Set(project.region.clone()),
            environment: Set(project.environment.to_string()),
            status: Set(project.status.to_string()),
            budget_limit: Set(project.budget_limit),
            tags: Set(serde_json::to_value(&project.tags).unwrap()),
            enabled: Set(project.enabled),
            created_at: Set(project.created_at.into()),
            updated_at: Set(project.updated_at.into()),
        };

        // Update using base repository
        let updated_model = self
            .base
            .update(active_model)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        tracing::info!(project_id = %id, "Updated project");
        Ok(updated_model.into())
    }

    async fn delete(&self, id: Uuid) -> ProjectResult<bool> {
        let rows_affected = self
            .base
            .delete_by_id(id)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        if rows_affected > 0 {
            tracing::info!(project_id = %id, "Deleted project");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists_by_name(&self, user_id: Uuid, name: &str) -> ProjectResult<bool> {
        let exists = entity::Entity::find()
            .filter(entity::Column::UserId.eq(user_id))
            .filter(entity::Column::Name.eq(name))
            .one(self.base.db())
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?
            .is_some();

        Ok(exists)
    }
}

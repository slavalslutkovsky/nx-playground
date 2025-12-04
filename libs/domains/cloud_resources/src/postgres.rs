use async_trait::async_trait;
use database::BaseRepository;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity,
    error::{CloudResourceError, CloudResourceResult},
    models::{CloudResource, CloudResourceFilter, CreateCloudResource, UpdateCloudResource},
    repository::CloudResourceRepository,
};

pub struct PgCloudResourceRepository {
    base: BaseRepository<entity::Entity>,
}

impl PgCloudResourceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            base: BaseRepository::new(db),
        }
    }
}

#[async_trait]
impl CloudResourceRepository for PgCloudResourceRepository {
    async fn create(&self, input: CreateCloudResource) -> CloudResourceResult<CloudResource> {
        // Convert CreateCloudResource to ActiveModel
        let active_model: entity::ActiveModel = input.into();

        // Insert using base repository
        let model = self
            .base
            .insert(active_model)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(model.into())
    }

    async fn get_by_id(&self, id: Uuid) -> CloudResourceResult<Option<CloudResource>> {
        let model = self
            .base
            .find_by_id(id)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(model.map(|m| m.into()))
    }

    async fn list(&self, filter: CloudResourceFilter) -> CloudResourceResult<Vec<CloudResource>> {
        let mut query = entity::Entity::find();

        // Apply filters
        if let Some(project_id) = filter.project_id {
            query = query.filter(entity::Column::ProjectId.eq(project_id));
        }

        if let Some(resource_type) = filter.resource_type {
            query = query.filter(entity::Column::ResourceType.eq(resource_type.to_string()));
        }

        if let Some(status) = filter.status {
            query = query.filter(entity::Column::Status.eq(status.to_string()));
        }

        if let Some(region) = filter.region {
            query = query.filter(entity::Column::Region.eq(region));
        }

        if let Some(enabled) = filter.enabled {
            query = query.filter(entity::Column::Enabled.eq(enabled));
        }

        // Apply pagination
        query = query
            .order_by_desc(entity::Column::CreatedAt)
            .limit(filter.limit as u64)
            .offset(filter.offset as u64);

        let models = query
            .all(self.base.db())
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    async fn list_by_project(&self, project_id: Uuid) -> CloudResourceResult<Vec<CloudResource>> {
        let models = entity::Entity::find()
            .filter(entity::Column::ProjectId.eq(project_id))
            .filter(entity::Column::DeletedAt.is_null())
            .order_by_desc(entity::Column::CreatedAt)
            .all(self.base.db())
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateCloudResource,
    ) -> CloudResourceResult<CloudResource> {
        // Fetch existing resource
        let model = self
            .base
            .find_by_id(id)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?
            .ok_or(CloudResourceError::NotFound(id))?;

        // Convert to domain model
        let mut resource: CloudResource = model.into();

        // Apply updates
        resource.apply_update(input);

        // Convert back to ActiveModel
        let active_model: entity::ActiveModel = entity::ActiveModel {
            id: Set(resource.id),
            project_id: Set(resource.project_id),
            name: Set(resource.name.clone()),
            resource_type: Set(resource.resource_type.to_string()),
            status: Set(resource.status.to_string()),
            region: Set(resource.region.clone()),
            configuration: Set(resource.configuration.clone()),
            cost_per_hour: Set(resource.cost_per_hour),
            monthly_cost_estimate: Set(resource.monthly_cost_estimate),
            tags: Set(serde_json::to_value(&resource.tags).unwrap()),
            enabled: Set(resource.enabled),
            created_at: Set(resource.created_at.into()),
            updated_at: Set(resource.updated_at.into()),
            deleted_at: Set(resource.deleted_at.map(|dt| dt.into())),
        };

        // Update using base repository
        let updated_model = self
            .base
            .update(active_model)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(updated_model.into())
    }

    async fn delete(&self, id: Uuid) -> CloudResourceResult<()> {
        let rows_affected = self
            .base
            .delete_by_id(id)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        if rows_affected == 0 {
            return Err(CloudResourceError::NotFound(id));
        }

        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> CloudResourceResult<()> {
        // Fetch existing resource
        let model = self
            .base
            .find_by_id(id)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?
            .ok_or(CloudResourceError::NotFound(id))?;

        // Convert to domain model and soft delete
        let mut resource: CloudResource = model.into();
        resource.soft_delete();

        // Update database
        let active_model: entity::ActiveModel = entity::ActiveModel {
            id: Set(resource.id),
            project_id: Set(resource.project_id),
            name: Set(resource.name.clone()),
            resource_type: Set(resource.resource_type.to_string()),
            status: Set(resource.status.to_string()),
            region: Set(resource.region.clone()),
            configuration: Set(resource.configuration.clone()),
            cost_per_hour: Set(resource.cost_per_hour),
            monthly_cost_estimate: Set(resource.monthly_cost_estimate),
            tags: Set(serde_json::to_value(&resource.tags).unwrap()),
            enabled: Set(resource.enabled),
            created_at: Set(resource.created_at.into()),
            updated_at: Set(resource.updated_at.into()),
            deleted_at: Set(resource.deleted_at.map(|dt| dt.into())),
        };

        self.base
            .update(active_model)
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn count_by_project(&self, project_id: Uuid) -> CloudResourceResult<usize> {
        let count = entity::Entity::find()
            .filter(entity::Column::ProjectId.eq(project_id))
            .filter(entity::Column::DeletedAt.is_null())
            .count(self.base.db())
            .await
            .map_err(|e| CloudResourceError::Internal(format!("Database error: {}", e)))?;

        Ok(count as usize)
    }
}

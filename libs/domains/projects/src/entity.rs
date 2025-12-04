use crate::models::{CloudProvider, Environment, ProjectStatus, Tag};
use core_proc_macros::SeaOrmResource;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

/// Sea-ORM Entity for Projects table
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub cloud_provider: CloudProvider,
    #[sea_orm(column_type = "Text")]
    pub region: String,
    pub environment: Environment,
    pub status: ProjectStatus,
    pub budget_limit: Option<f64>,
    pub tags: Json, // JSONB field
    pub enabled: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Conversion from Sea-ORM Model to domain Project
impl From<Model> for crate::models::Project {
    fn from(model: Model) -> Self {
        // Parse tags from JSON
        let tags: Vec<Tag> = serde_json::from_value(model.tags.clone()).unwrap_or_default();

        Self {
            id: model.id,
            name: model.name,
            user_id: model.user_id,
            description: model.description,
            cloud_provider: model.cloud_provider,
            region: model.region,
            environment: model.environment,
            status: model.status,
            budget_limit: model.budget_limit,
            tags,
            enabled: model.enabled,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

// Conversion from domain CreateProject to Sea-ORM ActiveModel
impl From<crate::models::CreateProject> for ActiveModel {
    fn from(input: crate::models::CreateProject) -> Self {
        let tags_json = serde_json::to_value(&input.tags).expect("Failed to serialize tags");

        ActiveModel {
            id: Set(Uuid::now_v7()),
            name: Set(input.name),
            user_id: Set(input.user_id),
            description: Set(input.description),
            cloud_provider: Set(input.cloud_provider),
            region: Set(input.region),
            environment: Set(input.environment),
            status: Set(ProjectStatus::Provisioning),
            budget_limit: Set(input.budget_limit),
            tags: Set(tags_json),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        }
    }
}

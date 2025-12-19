use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

// ===== Chat Sessions Entity =====

pub mod chat_sessions {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "finops_chat_sessions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub user_id: Option<Uuid>,
        #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
        pub title: Option<String>,
        #[sea_orm(column_type = "JsonBinary")]
        pub context: serde_json::Value,
        #[sea_orm(column_type = "String(StringLen::N(50))")]
        pub status: String,
        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::chat_messages::Entity")]
        Messages,
        #[sea_orm(has_many = "super::recommendations::Entity")]
        Recommendations,
    }

    impl Related<super::chat_messages::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Messages.def()
        }
    }

    impl Related<super::recommendations::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Recommendations.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for crate::models::ChatSession {
        fn from(model: Model) -> Self {
            let context: crate::models::ChatContext =
                serde_json::from_value(model.context).unwrap_or_default();

            Self {
                id: model.id,
                user_id: model.user_id,
                title: model.title,
                context,
                status: model.status.parse().unwrap_or_default(),
                created_at: model.created_at.into(),
                updated_at: model.updated_at.into(),
            }
        }
    }

    impl From<crate::models::CreateSession> for ActiveModel {
        fn from(input: crate::models::CreateSession) -> Self {
            let now = chrono::Utc::now();
            ActiveModel {
                id: Set(Uuid::now_v7()),
                user_id: Set(input.user_id),
                title: Set(input.title),
                context: Set(serde_json::to_value(input.context.unwrap_or_default())
                    .unwrap_or_default()),
                status: Set("active".to_string()),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            }
        }
    }
}

// ===== Chat Messages Entity =====

pub mod chat_messages {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "finops_chat_messages")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub session_id: Uuid,
        #[sea_orm(column_type = "String(StringLen::N(20))")]
        pub role: String,
        #[sea_orm(column_type = "Text", nullable)]
        pub content: Option<String>,
        #[sea_orm(column_type = "JsonBinary", nullable)]
        pub tool_calls: Option<serde_json::Value>,
        pub token_count: Option<i32>,
        pub latency_ms: Option<i32>,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::chat_sessions::Entity",
            from = "Column::SessionId",
            to = "super::chat_sessions::Column::Id"
        )]
        Session,
    }

    impl Related<super::chat_sessions::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Session.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for crate::models::ChatMessage {
        fn from(model: Model) -> Self {
            let tool_calls: Option<Vec<crate::models::ToolCallRecord>> = model
                .tool_calls
                .and_then(|v| serde_json::from_value(v).ok());

            Self {
                id: model.id,
                session_id: model.session_id,
                role: model.role.parse().unwrap_or_default(),
                content: model.content,
                tool_calls,
                token_count: model.token_count,
                latency_ms: model.latency_ms,
                created_at: model.created_at.into(),
            }
        }
    }
}

// ===== Cloud Accounts Entity =====

pub mod cloud_accounts {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "finops_cloud_accounts")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub user_id: Uuid,
        #[sea_orm(column_type = "String(StringLen::N(20))")]
        pub provider: String,
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub account_id: String,
        #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
        pub name: Option<String>,
        #[sea_orm(column_type = "VarBinary(StringLen::Max)", nullable)]
        pub credentials_encrypted: Option<Vec<u8>>,
        pub regions: Option<Vec<String>>,
        pub last_sync_at: Option<DateTimeWithTimeZone>,
        #[sea_orm(column_type = "String(StringLen::N(50))")]
        pub status: String,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::resources::Entity")]
        Resources,
    }

    impl Related<super::resources::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Resources.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for crate::models::CloudAccount {
        fn from(model: Model) -> Self {
            Self {
                id: model.id,
                user_id: model.user_id,
                provider: model.provider.parse().unwrap_or_default(),
                account_id: model.account_id,
                name: model.name,
                regions: model.regions.unwrap_or_default(),
                last_sync_at: model.last_sync_at.map(Into::into),
                status: model.status.parse().unwrap_or_default(),
                created_at: model.created_at.into(),
            }
        }
    }

    impl From<crate::models::CreateCloudAccount> for ActiveModel {
        fn from(input: crate::models::CreateCloudAccount) -> Self {
            let now = chrono::Utc::now();
            ActiveModel {
                id: Set(Uuid::now_v7()),
                user_id: Set(input.user_id),
                provider: Set(input.provider.to_string()),
                account_id: Set(input.account_id),
                name: Set(input.name),
                credentials_encrypted: Set(input.credentials.map(|c| c.into_bytes())),
                regions: Set(Some(input.regions)),
                last_sync_at: Set(None),
                status: Set("pending".to_string()),
                created_at: Set(now.into()),
            }
        }
    }
}

// ===== Resources Entity =====

pub mod resources {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "finops_resources")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub account_id: Uuid,
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub resource_id: String,
        #[sea_orm(column_type = "String(StringLen::N(100))")]
        pub resource_type: String,
        #[sea_orm(column_type = "String(StringLen::N(50))")]
        pub region: String,
        #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
        pub name: Option<String>,
        #[sea_orm(column_type = "JsonBinary")]
        pub specs: serde_json::Value,
        pub monthly_cost_cents: Option<i64>,
        #[sea_orm(column_type = "JsonBinary", nullable)]
        pub utilization: Option<serde_json::Value>,
        #[sea_orm(column_type = "JsonBinary", nullable)]
        pub tags: Option<serde_json::Value>,
        pub last_seen_at: DateTimeWithTimeZone,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::cloud_accounts::Entity",
            from = "Column::AccountId",
            to = "super::cloud_accounts::Column::Id"
        )]
        Account,
        #[sea_orm(has_many = "super::recommendations::Entity")]
        Recommendations,
    }

    impl Related<super::cloud_accounts::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Account.def()
        }
    }

    impl Related<super::recommendations::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Recommendations.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for crate::models::CloudResource {
        fn from(model: Model) -> Self {
            let specs: crate::models::ResourceSpecs =
                serde_json::from_value(model.specs).unwrap_or_default();
            let utilization: Option<crate::models::Utilization> =
                model.utilization.and_then(|v| serde_json::from_value(v).ok());
            let tags: Option<serde_json::Map<String, serde_json::Value>> =
                model.tags.and_then(|v| serde_json::from_value(v).ok());

            Self {
                id: model.id,
                account_id: model.account_id,
                resource_id: model.resource_id,
                resource_type: model.resource_type,
                region: model.region,
                name: model.name,
                specs,
                monthly_cost_cents: model.monthly_cost_cents,
                utilization,
                tags,
                last_seen_at: model.last_seen_at.into(),
                created_at: model.created_at.into(),
            }
        }
    }
}

// ===== Recommendations Entity =====

pub mod recommendations {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "finops_recommendations")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub session_id: Option<Uuid>,
        pub resource_id: Option<Uuid>,
        #[sea_orm(column_type = "String(StringLen::N(50))")]
        pub recommendation_type: String,
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub title: String,
        #[sea_orm(column_type = "Text")]
        pub description: String,
        pub current_cost_cents: Option<i64>,
        pub projected_cost_cents: Option<i64>,
        pub savings_cents: Option<i64>,
        pub confidence: Option<f32>,
        #[sea_orm(column_type = "JsonBinary", nullable)]
        pub details: Option<serde_json::Value>,
        #[sea_orm(column_type = "String(StringLen::N(50))")]
        pub status: String,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::chat_sessions::Entity",
            from = "Column::SessionId",
            to = "super::chat_sessions::Column::Id"
        )]
        Session,
        #[sea_orm(
            belongs_to = "super::resources::Entity",
            from = "Column::ResourceId",
            to = "super::resources::Column::Id"
        )]
        Resource,
    }

    impl Related<super::chat_sessions::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Session.def()
        }
    }

    impl Related<super::resources::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Resource.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for crate::models::Recommendation {
        fn from(model: Model) -> Self {
            let details: Option<crate::models::RecommendationDetails> =
                model.details.and_then(|v| serde_json::from_value(v).ok());

            Self {
                id: model.id,
                session_id: model.session_id,
                resource_id: model.resource_id,
                recommendation_type: model.recommendation_type.parse().unwrap_or_default(),
                title: model.title,
                description: model.description,
                current_cost_cents: model.current_cost_cents,
                projected_cost_cents: model.projected_cost_cents,
                savings_cents: model.savings_cents,
                confidence: model.confidence,
                details,
                status: model.status.parse().unwrap_or_default(),
                created_at: model.created_at.into(),
            }
        }
    }
}

// Re-export entities
pub use chat_messages::Entity as ChatMessagesEntity;
pub use chat_sessions::Entity as ChatSessionsEntity;
pub use cloud_accounts::Entity as CloudAccountsEntity;
pub use recommendations::Entity as RecommendationsEntity;
pub use resources::Entity as ResourcesEntity;

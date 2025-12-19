use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

use crate::entity::{
    chat_messages, chat_sessions, cloud_accounts, recommendations, resources, ChatMessagesEntity,
    ChatSessionsEntity, CloudAccountsEntity, RecommendationsEntity, ResourcesEntity,
};
use crate::error::{FinopsError, FinopsResult};
use crate::models::{
    ChatMessage, ChatSession, CloudAccount, CloudAccountStatus, CloudResource, CreateCloudAccount,
    CreateSession, MessageRole, Recommendation, RecommendationFilter, RecommendationStatus,
    ResourceFilter, SessionFilter, SessionStatus, ToolCallRecord,
};
use crate::repository::FinopsRepository;

/// PostgreSQL implementation of FinopsRepository
#[derive(Clone)]
pub struct PgFinopsRepository {
    db: DatabaseConnection,
}

impl PgFinopsRepository {
    /// Create a new PostgreSQL finops repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FinopsRepository for PgFinopsRepository {
    // ===== Chat Sessions =====

    async fn create_session(&self, input: CreateSession) -> FinopsResult<ChatSession> {
        let model: chat_sessions::ActiveModel = input.into();
        let result = model.insert(&self.db).await?.into();
        Ok(result)
    }

    async fn get_session(&self, id: Uuid) -> FinopsResult<Option<ChatSession>> {
        let result = ChatSessionsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(Into::into);
        Ok(result)
    }

    async fn list_sessions(&self, filter: SessionFilter) -> FinopsResult<Vec<ChatSession>> {
        let mut query = ChatSessionsEntity::find();

        if let Some(user_id) = filter.user_id {
            query = query.filter(chat_sessions::Column::UserId.eq(user_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(chat_sessions::Column::Status.eq(status.to_string()));
        }

        let results = query
            .order_by_desc(chat_sessions::Column::UpdatedAt)
            .offset(filter.offset as u64)
            .limit(filter.limit as u64)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn update_session_status(
        &self,
        id: Uuid,
        status: SessionStatus,
    ) -> FinopsResult<ChatSession> {
        let existing = ChatSessionsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| FinopsError::SessionNotFound(id.to_string()))?;

        let mut model: chat_sessions::ActiveModel = existing.into();
        model.status = Set(status.to_string());
        model.updated_at = Set(chrono::Utc::now().into());

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn update_session_title(&self, id: Uuid, title: &str) -> FinopsResult<ChatSession> {
        let existing = ChatSessionsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| FinopsError::SessionNotFound(id.to_string()))?;

        let mut model: chat_sessions::ActiveModel = existing.into();
        model.title = Set(Some(title.to_string()));
        model.updated_at = Set(chrono::Utc::now().into());

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn delete_session(&self, id: Uuid) -> FinopsResult<bool> {
        let result = ChatSessionsEntity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected > 0)
    }

    // ===== Chat Messages =====

    async fn save_message(
        &self,
        session_id: Uuid,
        role: MessageRole,
        content: Option<String>,
        tool_calls: Option<Vec<ToolCallRecord>>,
        token_count: Option<i32>,
        latency_ms: Option<i32>,
    ) -> FinopsResult<ChatMessage> {
        let now = chrono::Utc::now();
        let model = chat_messages::ActiveModel {
            id: Set(Uuid::now_v7()),
            session_id: Set(session_id),
            role: Set(role.to_string()),
            content: Set(content),
            tool_calls: Set(tool_calls.map(|tc| serde_json::to_value(tc).unwrap_or_default())),
            token_count: Set(token_count),
            latency_ms: Set(latency_ms),
            created_at: Set(now.into()),
        };

        let result = model.insert(&self.db).await?.into();
        Ok(result)
    }

    async fn get_messages(&self, session_id: Uuid) -> FinopsResult<Vec<ChatMessage>> {
        let results = ChatMessagesEntity::find()
            .filter(chat_messages::Column::SessionId.eq(session_id))
            .order_by_asc(chat_messages::Column::CreatedAt)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn get_messages_paginated(
        &self,
        session_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> FinopsResult<Vec<ChatMessage>> {
        let results = ChatMessagesEntity::find()
            .filter(chat_messages::Column::SessionId.eq(session_id))
            .order_by_asc(chat_messages::Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    // ===== Cloud Accounts =====

    async fn create_cloud_account(&self, input: CreateCloudAccount) -> FinopsResult<CloudAccount> {
        let model: cloud_accounts::ActiveModel = input.into();
        let result = model.insert(&self.db).await?.into();
        Ok(result)
    }

    async fn get_cloud_account(&self, id: Uuid) -> FinopsResult<Option<CloudAccount>> {
        let result = CloudAccountsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(Into::into);
        Ok(result)
    }

    async fn list_cloud_accounts(&self, user_id: Uuid) -> FinopsResult<Vec<CloudAccount>> {
        let results = CloudAccountsEntity::find()
            .filter(cloud_accounts::Column::UserId.eq(user_id))
            .order_by_desc(cloud_accounts::Column::CreatedAt)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn update_cloud_account_sync(&self, id: Uuid) -> FinopsResult<CloudAccount> {
        let existing = CloudAccountsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| FinopsError::AccountNotFound(id.to_string()))?;

        let mut model: cloud_accounts::ActiveModel = existing.into();
        model.last_sync_at = Set(Some(chrono::Utc::now().into()));

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn update_cloud_account_status(
        &self,
        id: Uuid,
        status: CloudAccountStatus,
    ) -> FinopsResult<CloudAccount> {
        let existing = CloudAccountsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| FinopsError::AccountNotFound(id.to_string()))?;

        let mut model: cloud_accounts::ActiveModel = existing.into();
        model.status = Set(status.to_string());

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn delete_cloud_account(&self, id: Uuid) -> FinopsResult<bool> {
        let result = CloudAccountsEntity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected > 0)
    }

    // ===== Resources =====

    async fn upsert_resource(&self, resource: CloudResource) -> FinopsResult<CloudResource> {
        // Check if resource exists by account_id + resource_id
        let existing = ResourcesEntity::find()
            .filter(resources::Column::AccountId.eq(resource.account_id))
            .filter(resources::Column::ResourceId.eq(&resource.resource_id))
            .one(&self.db)
            .await?;

        let now = chrono::Utc::now();

        if let Some(existing) = existing {
            // Update existing resource
            let mut model: resources::ActiveModel = existing.into();
            model.name = Set(resource.name);
            model.specs = Set(serde_json::to_value(&resource.specs).unwrap_or_default());
            model.monthly_cost_cents = Set(resource.monthly_cost_cents);
            model.utilization =
                Set(resource.utilization.map(|u| serde_json::to_value(u).unwrap_or_default()));
            model.tags = Set(resource.tags.map(serde_json::Value::Object));
            model.last_seen_at = Set(now.into());

            let result = model.update(&self.db).await?.into();
            Ok(result)
        } else {
            // Create new resource
            let model = resources::ActiveModel {
                id: Set(Uuid::now_v7()),
                account_id: Set(resource.account_id),
                resource_id: Set(resource.resource_id),
                resource_type: Set(resource.resource_type),
                region: Set(resource.region),
                name: Set(resource.name),
                specs: Set(serde_json::to_value(&resource.specs).unwrap_or_default()),
                monthly_cost_cents: Set(resource.monthly_cost_cents),
                utilization: Set(resource
                    .utilization
                    .map(|u| serde_json::to_value(u).unwrap_or_default())),
                tags: Set(resource.tags.map(serde_json::Value::Object)),
                last_seen_at: Set(now.into()),
                created_at: Set(now.into()),
            };

            let result = model.insert(&self.db).await?.into();
            Ok(result)
        }
    }

    async fn upsert_resources(&self, resources: Vec<CloudResource>) -> FinopsResult<usize> {
        let mut count = 0;
        for resource in resources {
            self.upsert_resource(resource).await?;
            count += 1;
        }
        Ok(count)
    }

    async fn get_resource(&self, id: Uuid) -> FinopsResult<Option<CloudResource>> {
        let result = ResourcesEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(Into::into);
        Ok(result)
    }

    async fn list_resources(&self, filter: ResourceFilter) -> FinopsResult<Vec<CloudResource>> {
        let mut query = ResourcesEntity::find();

        if let Some(account_id) = filter.account_id {
            query = query.filter(resources::Column::AccountId.eq(account_id));
        }

        if let Some(resource_type) = filter.resource_type {
            query = query.filter(resources::Column::ResourceType.eq(resource_type));
        }

        if let Some(region) = filter.region {
            query = query.filter(resources::Column::Region.eq(region));
        }

        if let Some(min_cost) = filter.min_cost {
            query = query.filter(resources::Column::MonthlyCostCents.gte(min_cost));
        }

        let results = query
            .order_by_desc(resources::Column::MonthlyCostCents)
            .offset(filter.offset as u64)
            .limit(filter.limit as u64)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn get_resources_by_account(&self, account_id: Uuid) -> FinopsResult<Vec<CloudResource>> {
        let results = ResourcesEntity::find()
            .filter(resources::Column::AccountId.eq(account_id))
            .order_by_desc(resources::Column::MonthlyCostCents)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn delete_stale_resources(
        &self,
        account_id: Uuid,
        before: chrono::DateTime<chrono::Utc>,
    ) -> FinopsResult<usize> {
        let before_tz: chrono::DateTime<chrono::FixedOffset> = before.into();
        let result = ResourcesEntity::delete_many()
            .filter(resources::Column::AccountId.eq(account_id))
            .filter(resources::Column::LastSeenAt.lt(before_tz))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected as usize)
    }

    // ===== Recommendations =====

    async fn create_recommendation(
        &self,
        recommendation: Recommendation,
    ) -> FinopsResult<Recommendation> {
        let now = chrono::Utc::now();
        let model = recommendations::ActiveModel {
            id: Set(Uuid::now_v7()),
            session_id: Set(recommendation.session_id),
            resource_id: Set(recommendation.resource_id),
            recommendation_type: Set(recommendation.recommendation_type.to_string()),
            title: Set(recommendation.title),
            description: Set(recommendation.description),
            current_cost_cents: Set(recommendation.current_cost_cents),
            projected_cost_cents: Set(recommendation.projected_cost_cents),
            savings_cents: Set(recommendation.savings_cents),
            confidence: Set(recommendation.confidence),
            details: Set(recommendation
                .details
                .map(|d| serde_json::to_value(d).unwrap_or_default())),
            status: Set(recommendation.status.to_string()),
            created_at: Set(now.into()),
        };

        let result = model.insert(&self.db).await?.into();
        Ok(result)
    }

    async fn get_recommendation(&self, id: Uuid) -> FinopsResult<Option<Recommendation>> {
        let result = RecommendationsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(Into::into);
        Ok(result)
    }

    async fn list_recommendations(
        &self,
        filter: RecommendationFilter,
    ) -> FinopsResult<Vec<Recommendation>> {
        let mut query = RecommendationsEntity::find();

        if let Some(session_id) = filter.session_id {
            query = query.filter(recommendations::Column::SessionId.eq(session_id));
        }

        if let Some(resource_id) = filter.resource_id {
            query = query.filter(recommendations::Column::ResourceId.eq(resource_id));
        }

        if let Some(recommendation_type) = filter.recommendation_type {
            query = query.filter(
                recommendations::Column::RecommendationType.eq(recommendation_type.to_string()),
            );
        }

        if let Some(status) = filter.status {
            query = query.filter(recommendations::Column::Status.eq(status.to_string()));
        }

        let results = query
            .order_by_desc(recommendations::Column::SavingsCents)
            .offset(filter.offset as u64)
            .limit(filter.limit as u64)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn update_recommendation_status(
        &self,
        id: Uuid,
        status: RecommendationStatus,
    ) -> FinopsResult<Recommendation> {
        let existing = RecommendationsEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| FinopsError::ResourceNotFound(id.to_string()))?;

        let mut model: recommendations::ActiveModel = existing.into();
        model.status = Set(status.to_string());

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn get_recommendations_for_resource(
        &self,
        resource_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>> {
        let results = RecommendationsEntity::find()
            .filter(recommendations::Column::ResourceId.eq(resource_id))
            .order_by_desc(recommendations::Column::SavingsCents)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }

    async fn get_recommendations_for_session(
        &self,
        session_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>> {
        let results = RecommendationsEntity::find()
            .filter(recommendations::Column::SessionId.eq(session_id))
            .order_by_desc(recommendations::Column::SavingsCents)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }
}

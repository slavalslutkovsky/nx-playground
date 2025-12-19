use async_trait::async_trait;
use uuid::Uuid;

use crate::error::FinopsResult;
use crate::models::{
    ChatMessage, ChatSession, CloudAccount, CloudResource, CreateCloudAccount, CreateSession,
    MessageRole, Recommendation, RecommendationFilter, RecommendationStatus, ResourceFilter,
    SessionFilter, SessionStatus, ToolCallRecord,
};

/// Repository trait for FinOps persistence
///
/// This trait defines the data access interface for the FinOps domain.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FinopsRepository: Send + Sync {
    // ===== Chat Sessions =====

    /// Create a new chat session
    async fn create_session(&self, input: CreateSession) -> FinopsResult<ChatSession>;

    /// Get a chat session by ID
    async fn get_session(&self, id: Uuid) -> FinopsResult<Option<ChatSession>>;

    /// List chat sessions with optional filters
    async fn list_sessions(&self, filter: SessionFilter) -> FinopsResult<Vec<ChatSession>>;

    /// Update session status
    async fn update_session_status(
        &self,
        id: Uuid,
        status: SessionStatus,
    ) -> FinopsResult<ChatSession>;

    /// Update session title
    async fn update_session_title(&self, id: Uuid, title: &str) -> FinopsResult<ChatSession>;

    /// Delete a chat session
    async fn delete_session(&self, id: Uuid) -> FinopsResult<bool>;

    // ===== Chat Messages =====

    /// Save a chat message
    async fn save_message(
        &self,
        session_id: Uuid,
        role: MessageRole,
        content: Option<String>,
        tool_calls: Option<Vec<ToolCallRecord>>,
        token_count: Option<i32>,
        latency_ms: Option<i32>,
    ) -> FinopsResult<ChatMessage>;

    /// Get messages for a session
    async fn get_messages(&self, session_id: Uuid) -> FinopsResult<Vec<ChatMessage>>;

    /// Get messages for a session with pagination
    async fn get_messages_paginated(
        &self,
        session_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> FinopsResult<Vec<ChatMessage>>;

    // ===== Cloud Accounts =====

    /// Connect a cloud account
    async fn create_cloud_account(&self, input: CreateCloudAccount) -> FinopsResult<CloudAccount>;

    /// Get a cloud account by ID
    async fn get_cloud_account(&self, id: Uuid) -> FinopsResult<Option<CloudAccount>>;

    /// List cloud accounts for a user
    async fn list_cloud_accounts(&self, user_id: Uuid) -> FinopsResult<Vec<CloudAccount>>;

    /// Update cloud account sync timestamp
    async fn update_cloud_account_sync(&self, id: Uuid) -> FinopsResult<CloudAccount>;

    /// Update cloud account status
    async fn update_cloud_account_status(
        &self,
        id: Uuid,
        status: crate::models::CloudAccountStatus,
    ) -> FinopsResult<CloudAccount>;

    /// Delete a cloud account
    async fn delete_cloud_account(&self, id: Uuid) -> FinopsResult<bool>;

    // ===== Resources =====

    /// Upsert a cloud resource
    async fn upsert_resource(&self, resource: CloudResource) -> FinopsResult<CloudResource>;

    /// Upsert multiple cloud resources
    async fn upsert_resources(&self, resources: Vec<CloudResource>) -> FinopsResult<usize>;

    /// Get a resource by ID
    async fn get_resource(&self, id: Uuid) -> FinopsResult<Option<CloudResource>>;

    /// List resources with filters
    async fn list_resources(&self, filter: ResourceFilter) -> FinopsResult<Vec<CloudResource>>;

    /// Get resources by account
    async fn get_resources_by_account(&self, account_id: Uuid) -> FinopsResult<Vec<CloudResource>>;

    /// Delete resources not seen since timestamp
    async fn delete_stale_resources(
        &self,
        account_id: Uuid,
        before: chrono::DateTime<chrono::Utc>,
    ) -> FinopsResult<usize>;

    // ===== Recommendations =====

    /// Create a recommendation
    async fn create_recommendation(
        &self,
        recommendation: Recommendation,
    ) -> FinopsResult<Recommendation>;

    /// Get a recommendation by ID
    async fn get_recommendation(&self, id: Uuid) -> FinopsResult<Option<Recommendation>>;

    /// List recommendations with filters
    async fn list_recommendations(
        &self,
        filter: RecommendationFilter,
    ) -> FinopsResult<Vec<Recommendation>>;

    /// Update recommendation status
    async fn update_recommendation_status(
        &self,
        id: Uuid,
        status: RecommendationStatus,
    ) -> FinopsResult<Recommendation>;

    /// Get recommendations for a resource
    async fn get_recommendations_for_resource(
        &self,
        resource_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>>;

    /// Get recommendations for a session
    async fn get_recommendations_for_session(
        &self,
        session_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>>;
}

use std::sync::Arc;
use uuid::Uuid;

use crate::error::{FinopsError, FinopsResult};
use crate::models::{
    ChatContext, ChatMessage, ChatSession, CloudAccount, CloudAccountStatus,
    CloudResource, CreateCloudAccount, CreateSession, MessageRole, Recommendation,
    RecommendationFilter, RecommendationStatus, ResourceFilter, SessionFilter, SessionStatus,
    ToolCallRecord,
};
use crate::repository::FinopsRepository;

/// Service for managing FinOps chat and resources
#[derive(Clone)]
pub struct FinopsService<R: FinopsRepository> {
    repository: Arc<R>,
}

impl<R: FinopsRepository> FinopsService<R> {
    /// Create a new finops service
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    // ===== Session Management =====

    /// Get or create a chat session
    pub async fn get_or_create_session(
        &self,
        session_id: Option<Uuid>,
        user_id: Option<Uuid>,
        context: Option<ChatContext>,
    ) -> FinopsResult<ChatSession> {
        if let Some(id) = session_id {
            if let Some(session) = self.repository.get_session(id).await? {
                return Ok(session);
            }
        }

        // Create new session
        let input = CreateSession {
            user_id,
            title: None,
            context,
        };
        self.repository.create_session(input).await
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: Uuid) -> FinopsResult<ChatSession> {
        self.repository
            .get_session(id)
            .await?
            .ok_or_else(|| FinopsError::SessionNotFound(id.to_string()))
    }

    /// List sessions for a user
    pub async fn list_sessions(&self, filter: SessionFilter) -> FinopsResult<Vec<ChatSession>> {
        self.repository.list_sessions(filter).await
    }

    /// Archive a session
    pub async fn archive_session(&self, id: Uuid) -> FinopsResult<ChatSession> {
        self.repository
            .update_session_status(id, SessionStatus::Archived)
            .await
    }

    /// Complete a session
    pub async fn complete_session(&self, id: Uuid) -> FinopsResult<ChatSession> {
        self.repository
            .update_session_status(id, SessionStatus::Completed)
            .await
    }

    /// Update session title (auto-generate from first message if not set)
    pub async fn update_session_title(&self, id: Uuid, title: &str) -> FinopsResult<ChatSession> {
        self.repository.update_session_title(id, title).await
    }

    /// Delete a session
    pub async fn delete_session(&self, id: Uuid) -> FinopsResult<bool> {
        self.repository.delete_session(id).await
    }

    // ===== Message Management =====

    /// Save a user message
    pub async fn save_user_message(
        &self,
        session_id: Uuid,
        content: &str,
    ) -> FinopsResult<ChatMessage> {
        self.repository
            .save_message(session_id, MessageRole::User, Some(content.to_string()), None, None, None)
            .await
    }

    /// Save an assistant message
    pub async fn save_assistant_message(
        &self,
        session_id: Uuid,
        content: &str,
        tool_calls: Option<Vec<ToolCallRecord>>,
        token_count: Option<i32>,
        latency_ms: Option<i32>,
    ) -> FinopsResult<ChatMessage> {
        self.repository
            .save_message(
                session_id,
                MessageRole::Assistant,
                Some(content.to_string()),
                tool_calls,
                token_count,
                latency_ms,
            )
            .await
    }

    /// Save a tool message
    pub async fn save_tool_message(
        &self,
        session_id: Uuid,
        tool_calls: Vec<ToolCallRecord>,
    ) -> FinopsResult<ChatMessage> {
        self.repository
            .save_message(
                session_id,
                MessageRole::Tool,
                None,
                Some(tool_calls),
                None,
                None,
            )
            .await
    }

    /// Get conversation history
    pub async fn get_conversation_history(
        &self,
        session_id: Uuid,
    ) -> FinopsResult<Vec<ChatMessage>> {
        self.repository.get_messages(session_id).await
    }

    /// Get conversation history with pagination
    pub async fn get_conversation_history_paginated(
        &self,
        session_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> FinopsResult<Vec<ChatMessage>> {
        self.repository
            .get_messages_paginated(session_id, limit, offset)
            .await
    }

    /// Build context string from conversation history for the agent
    pub async fn build_conversation_context(&self, session_id: Uuid) -> FinopsResult<String> {
        let messages = self.get_conversation_history(session_id).await?;

        let context = messages
            .into_iter()
            .filter_map(|msg| {
                let role = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::System => "System",
                    MessageRole::Tool => return None, // Skip tool messages in context
                };
                msg.content.map(|content| format!("{}: {}", role, content))
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(context)
    }

    // ===== Cloud Account Management =====

    /// Connect a cloud account
    pub async fn connect_cloud_account(
        &self,
        input: CreateCloudAccount,
    ) -> FinopsResult<CloudAccount> {
        self.repository.create_cloud_account(input).await
    }

    /// Get a cloud account
    pub async fn get_cloud_account(&self, id: Uuid) -> FinopsResult<CloudAccount> {
        self.repository
            .get_cloud_account(id)
            .await?
            .ok_or_else(|| FinopsError::AccountNotFound(id.to_string()))
    }

    /// List cloud accounts for a user
    pub async fn list_cloud_accounts(&self, user_id: Uuid) -> FinopsResult<Vec<CloudAccount>> {
        self.repository.list_cloud_accounts(user_id).await
    }

    /// Mark account as connected after successful sync
    pub async fn mark_account_connected(&self, id: Uuid) -> FinopsResult<CloudAccount> {
        let account = self
            .repository
            .update_cloud_account_status(id, CloudAccountStatus::Connected)
            .await?;
        self.repository.update_cloud_account_sync(id).await?;
        Ok(account)
    }

    /// Mark account as error
    pub async fn mark_account_error(&self, id: Uuid) -> FinopsResult<CloudAccount> {
        self.repository
            .update_cloud_account_status(id, CloudAccountStatus::Error)
            .await
    }

    /// Disconnect a cloud account
    pub async fn disconnect_cloud_account(&self, id: Uuid) -> FinopsResult<bool> {
        self.repository.delete_cloud_account(id).await
    }

    // ===== Resource Management =====

    /// Sync resources from cloud account
    pub async fn sync_resources(
        &self,
        account_id: Uuid,
        resources: Vec<CloudResource>,
    ) -> FinopsResult<usize> {
        let count = self.repository.upsert_resources(resources).await?;

        // Clean up stale resources (not seen in last sync)
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        self.repository
            .delete_stale_resources(account_id, cutoff)
            .await?;

        // Update sync timestamp
        self.repository.update_cloud_account_sync(account_id).await?;

        Ok(count)
    }

    /// Get a resource
    pub async fn get_resource(&self, id: Uuid) -> FinopsResult<CloudResource> {
        self.repository
            .get_resource(id)
            .await?
            .ok_or_else(|| FinopsError::ResourceNotFound(id.to_string()))
    }

    /// List resources with filters
    pub async fn list_resources(&self, filter: ResourceFilter) -> FinopsResult<Vec<CloudResource>> {
        self.repository.list_resources(filter).await
    }

    /// Get resources for an account
    pub async fn get_resources_by_account(
        &self,
        account_id: Uuid,
    ) -> FinopsResult<Vec<CloudResource>> {
        self.repository.get_resources_by_account(account_id).await
    }

    // ===== Recommendation Management =====

    /// Create a recommendation
    pub async fn create_recommendation(
        &self,
        recommendation: Recommendation,
    ) -> FinopsResult<Recommendation> {
        self.repository.create_recommendation(recommendation).await
    }

    /// Get a recommendation
    pub async fn get_recommendation(&self, id: Uuid) -> FinopsResult<Recommendation> {
        self.repository
            .get_recommendation(id)
            .await?
            .ok_or_else(|| FinopsError::ResourceNotFound(id.to_string()))
    }

    /// List recommendations
    pub async fn list_recommendations(
        &self,
        filter: RecommendationFilter,
    ) -> FinopsResult<Vec<Recommendation>> {
        self.repository.list_recommendations(filter).await
    }

    /// Approve a recommendation
    pub async fn approve_recommendation(&self, id: Uuid) -> FinopsResult<Recommendation> {
        self.repository
            .update_recommendation_status(id, RecommendationStatus::Approved)
            .await
    }

    /// Dismiss a recommendation
    pub async fn dismiss_recommendation(&self, id: Uuid) -> FinopsResult<Recommendation> {
        self.repository
            .update_recommendation_status(id, RecommendationStatus::Dismissed)
            .await
    }

    /// Mark recommendation as applied
    pub async fn mark_recommendation_applied(&self, id: Uuid) -> FinopsResult<Recommendation> {
        self.repository
            .update_recommendation_status(id, RecommendationStatus::Applied)
            .await
    }

    /// Mark recommendation as failed
    pub async fn mark_recommendation_failed(&self, id: Uuid) -> FinopsResult<Recommendation> {
        self.repository
            .update_recommendation_status(id, RecommendationStatus::Failed)
            .await
    }

    /// Get recommendations for a resource
    pub async fn get_recommendations_for_resource(
        &self,
        resource_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>> {
        self.repository
            .get_recommendations_for_resource(resource_id)
            .await
    }

    /// Get recommendations for a session
    pub async fn get_recommendations_for_session(
        &self,
        session_id: Uuid,
    ) -> FinopsResult<Vec<Recommendation>> {
        self.repository
            .get_recommendations_for_session(session_id)
            .await
    }
}

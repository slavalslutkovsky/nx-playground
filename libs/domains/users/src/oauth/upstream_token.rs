//! Upstream OAuth tokens storage
//!
//! Stores OAuth tokens from upstream providers (e.g., Google via WorkOS)
//! that can be used to access external APIs like GCP.

use crate::error::{UserError, UserResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Upstream OAuth token for accessing external APIs (e.g., GCP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamOAuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    /// The upstream provider (google, github, etc.)
    pub provider: String,
    /// The auth source that provided these tokens (workos, direct, etc.)
    pub auth_source: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Parameters for creating/updating upstream OAuth token
#[derive(Debug, Clone)]
pub struct UpsertUpstreamTokenParams {
    pub user_id: Uuid,
    pub provider: String,
    pub auth_source: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scopes: Option<Vec<String>>,
}

/// Repository for upstream OAuth token operations
#[async_trait]
pub trait UpstreamOAuthTokenRepository: Send + Sync + Clone {
    /// Upsert (insert or update) upstream token for a user + provider
    async fn upsert(&self, params: UpsertUpstreamTokenParams) -> UserResult<UpstreamOAuthToken>;

    /// Find token by user_id and provider
    async fn find_by_user_and_provider(
        &self,
        user_id: Uuid,
        provider: &str,
    ) -> UserResult<Option<UpstreamOAuthToken>>;

    /// Find all tokens for a user
    async fn find_by_user_id(&self, user_id: Uuid) -> UserResult<Vec<UpstreamOAuthToken>>;

    /// Delete token by user_id and provider
    async fn delete_by_user_and_provider(&self, user_id: Uuid, provider: &str) -> UserResult<bool>;

    /// Update tokens (for refresh)
    async fn update_tokens(
        &self,
        user_id: Uuid,
        provider: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: Option<u64>,
    ) -> UserResult<()>;
}

/// PostgreSQL implementation of UpstreamOAuthTokenRepository
#[derive(Clone)]
pub struct PostgresUpstreamOAuthTokenRepository {
    db: sea_orm::DatabaseConnection,
}

impl PostgresUpstreamOAuthTokenRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[derive(Debug, FromQueryResult)]
struct UpstreamOAuthTokenRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    auth_source: String,
    access_token: String,
    refresh_token: Option<String>,
    token_expires_at: Option<DateTime<Utc>>,
    scopes: Option<Vec<String>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UpstreamOAuthTokenRow> for UpstreamOAuthToken {
    fn from(row: UpstreamOAuthTokenRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            provider: row.provider,
            auth_source: row.auth_source,
            access_token: row.access_token,
            refresh_token: row.refresh_token,
            token_expires_at: row.token_expires_at,
            scopes: row.scopes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl UpstreamOAuthTokenRepository for PostgresUpstreamOAuthTokenRepository {
    async fn upsert(&self, params: UpsertUpstreamTokenParams) -> UserResult<UpstreamOAuthToken> {
        let now = Utc::now();
        let token_expires_at =
            params
                .expires_in
                .map(|secs| now + chrono::Duration::seconds(secs as i64));

        // Use ON CONFLICT to upsert
        let sql = r#"
            INSERT INTO upstream_oauth_tokens (
                id, user_id, provider, auth_source, access_token, refresh_token,
                token_expires_at, scopes, created_at, updated_at
            ) VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, provider) DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = COALESCE(EXCLUDED.refresh_token, upstream_oauth_tokens.refresh_token),
                token_expires_at = EXCLUDED.token_expires_at,
                scopes = COALESCE(EXCLUDED.scopes, upstream_oauth_tokens.scopes),
                auth_source = EXCLUDED.auth_source,
                updated_at = EXCLUDED.updated_at
            RETURNING *
        "#;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                params.user_id.into(),
                params.provider.into(),
                params.auth_source.into(),
                params.access_token.into(),
                params.refresh_token.into(),
                token_expires_at.into(),
                params.scopes.into(),
                now.into(),
                now.into(),
            ],
        );

        let row = UpstreamOAuthTokenRow::find_by_statement(stmt)
            .one(&self.db)
            .await
            .map_err(|e| UserError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| {
                UserError::Internal("Failed to upsert upstream OAuth token".to_string())
            })?;

        Ok(row.into())
    }

    async fn find_by_user_and_provider(
        &self,
        user_id: Uuid,
        provider: &str,
    ) -> UserResult<Option<UpstreamOAuthToken>> {
        let sql = "SELECT * FROM upstream_oauth_tokens WHERE user_id = $1 AND provider = $2";

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [user_id.into(), provider.into()],
        );

        let row = UpstreamOAuthTokenRow::find_by_statement(stmt)
            .one(&self.db)
            .await
            .map_err(|e| UserError::Internal(format!("Database error: {}", e)))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> UserResult<Vec<UpstreamOAuthToken>> {
        let sql = "SELECT * FROM upstream_oauth_tokens WHERE user_id = $1 ORDER BY created_at DESC";

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into()]);

        let rows = UpstreamOAuthTokenRow::find_by_statement(stmt)
            .all(&self.db)
            .await
            .map_err(|e| UserError::Internal(format!("Database error: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_by_user_and_provider(&self, user_id: Uuid, provider: &str) -> UserResult<bool> {
        let sql = "DELETE FROM upstream_oauth_tokens WHERE user_id = $1 AND provider = $2";

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [user_id.into(), provider.into()],
        );

        let result = self
            .db
            .execute_raw(stmt)
            .await
            .map_err(|e| UserError::Internal(format!("Database error: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_tokens(
        &self,
        user_id: Uuid,
        provider: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: Option<u64>,
    ) -> UserResult<()> {
        let now = Utc::now();
        let token_expires_at =
            expires_in.map(|secs| now + chrono::Duration::seconds(secs as i64));

        let sql = r#"
            UPDATE upstream_oauth_tokens
            SET access_token = $3,
                refresh_token = COALESCE($4, refresh_token),
                token_expires_at = $5,
                updated_at = $6
            WHERE user_id = $1 AND provider = $2
        "#;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                user_id.into(),
                provider.into(),
                access_token.into(),
                refresh_token.into(),
                token_expires_at.into(),
                now.into(),
            ],
        );

        self.db
            .execute_raw(stmt)
            .await
            .map_err(|e| UserError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}

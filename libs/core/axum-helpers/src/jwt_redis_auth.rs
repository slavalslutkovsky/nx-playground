use crate::redis_auth_store::RedisAuthStore;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT token time-to-live constants
pub const ACCESS_TOKEN_TTL: i64 = 900; // 15 minutes
pub const REFRESH_TOKEN_TTL: i64 = 604800; // 7 days

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub name: String,       // User name
    pub roles: Vec<String>, // User roles
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID (for whitelist/blacklist)
}

/// Hybrid JWT + Redis authentication
/// Combines stateless JWT tokens with Redis-backed whitelist/blacklist
#[derive(Clone)]
pub struct JwtRedisAuth {
    secret: String,
    store: RedisAuthStore,
}

impl JwtRedisAuth {
    /// Create new JWT + Redis auth instance
    pub fn new(manager: ConnectionManager, jwt_secret: Option<&str>) -> eyre::Result<Self> {
        let store = RedisAuthStore::new(manager);

        let secret = jwt_secret
            .map(String::from)
            .or_else(|| std::env::var("JWT_SECRET").ok())
            .unwrap_or_else(|| "default-secret-key-change-me-in-production".to_string());

        tracing::info!("JWT + Redis auth initialized");
        Ok(Self { secret, store })
    }

    /// Create access token (15 min)
    pub fn create_access_token(
        &self,
        user_id: &str,
        email: &str,
        name: &str,
        roles: &[String],
    ) -> eyre::Result<String> {
        self.create_token(user_id, email, name, roles, ACCESS_TOKEN_TTL)
    }

    /// Create refresh token (7 days)
    pub fn create_refresh_token(
        &self,
        user_id: &str,
        email: &str,
        name: &str,
        roles: &[String],
    ) -> eyre::Result<String> {
        self.create_token(user_id, email, name, roles, REFRESH_TOKEN_TTL)
    }

    /// Create JWT token with specified TTL
    fn create_token(
        &self,
        user_id: &str,
        email: &str,
        name: &str,
        roles: &[String],
        ttl_seconds: i64,
    ) -> eyre::Result<String> {
        let now = Utc::now();
        let exp = (now + Duration::seconds(ttl_seconds)).timestamp();
        let iat = now.timestamp();
        let jti = Uuid::new_v4().to_string();

        let claims = JwtClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            roles: roles.to_vec(),
            exp,
            iat,
            jti,
        };

        let header = Header {
            alg: jsonwebtoken::Algorithm::HS256,
            ..Default::default()
        };

        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// Verify JWT token signature and decode claims
    pub fn verify_token(&self, token: &str) -> eyre::Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Add token to whitelist in Redis
    pub async fn whitelist_token(&self, jti: &str, user_id: &str, ttl: u64) -> eyre::Result<()> {
        let mut store = self.store.clone();
        store
            .store_jwt_whitelist(jti, user_id, ttl)
            .await
            .map_err(|e| eyre::eyre!("Failed to whitelist token: {}", e))?;
        Ok(())
    }

    /// Check if token is whitelisted
    pub async fn is_token_whitelisted(&self, jti: &str) -> eyre::Result<bool> {
        let mut store = self.store.clone();
        store
            .check_jwt_whitelist(jti)
            .await
            .map_err(|e| eyre::eyre!("Failed to check whitelist: {}", e))
    }

    /// Add token to blacklist in Redis
    pub async fn blacklist_token(&self, jti: &str, ttl: u64) -> eyre::Result<()> {
        let mut store = self.store.clone();
        store
            .blacklist_jwt(jti, ttl)
            .await
            .map_err(|e| eyre::eyre!("Failed to blacklist token: {}", e))?;
        Ok(())
    }

    /// Check if token is blacklisted
    pub async fn is_token_blacklisted(&self, jti: &str) -> eyre::Result<bool> {
        let mut store = self.store.clone();
        store
            .check_jwt_blacklist(jti)
            .await
            .map_err(|e| eyre::eyre!("Failed to check blacklist: {}", e))
    }

    /// Remove token from whitelist (on logout/refresh)
    pub async fn revoke_token(&self, jti: &str) -> eyre::Result<()> {
        let mut store = self.store.clone();
        store
            .revoke_jwt_whitelist(jti)
            .await
            .map_err(|e| eyre::eyre!("Failed to revoke token: {}", e))?;
        Ok(())
    }
}

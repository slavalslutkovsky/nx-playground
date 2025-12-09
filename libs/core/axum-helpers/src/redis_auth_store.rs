use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};

/// Redis-backed store for JWT authentication
/// Handles whitelist/blacklist for tokens and CSRF tokens
#[derive(Clone)]
pub struct RedisAuthStore {
    client: ConnectionManager,
}

impl RedisAuthStore {
    pub fn new(manager: ConnectionManager) -> Self {
        tracing::info!("Redis auth store initialized");
        Self { client: manager }
    }

    /// Store JWT in whitelist with TTL
    pub async fn store_jwt_whitelist(
        &mut self,
        jti: &str,
        user_id: &str,
        ttl_seconds: u64,
    ) -> RedisResult<()> {
        let key = format!("jwt:whitelist:{}", jti);
        self.client
            .set_ex::<_, _, ()>(&key, user_id, ttl_seconds)
            .await?;
        Ok(())
    }

    /// Check if JWT is in whitelist
    pub async fn check_jwt_whitelist(&mut self, jti: &str) -> RedisResult<bool> {
        let key = format!("jwt:whitelist:{}", jti);
        let exists: bool = self.client.exists(&key).await?;
        Ok(exists)
    }

    /// Add JWT to blacklist with TTL
    pub async fn blacklist_jwt(&mut self, jti: &str, ttl_seconds: u64) -> RedisResult<()> {
        let key = format!("jwt:blacklist:{}", jti);
        self.client
            .set_ex::<_, _, ()>(&key, "1", ttl_seconds)
            .await?;
        Ok(())
    }

    /// Check if JWT is in blacklist
    pub async fn check_jwt_blacklist(&mut self, jti: &str) -> RedisResult<bool> {
        let key = format!("jwt:blacklist:{}", jti);
        let exists: bool = self.client.exists(&key).await?;
        Ok(exists)
    }

    /// Remove JWT from whitelist (on logout or refresh)
    pub async fn revoke_jwt_whitelist(&mut self, jti: &str) -> RedisResult<()> {
        let key = format!("jwt:whitelist:{}", jti);
        self.client.del::<_, ()>(&key).await?;
        Ok(())
    }

    /// Store CSRF token with TTL
    pub async fn store_csrf_token(&mut self, token: &str, ttl_seconds: u64) -> RedisResult<()> {
        let key = format!("csrf:{}", token);
        let _: () = self.client.set_ex(&key, "1", ttl_seconds).await?;
        Ok(())
    }

    /// Validate and consume CSRF token (one-time use)
    pub async fn validate_and_remove_csrf_token(&mut self, token: &str) -> RedisResult<bool> {
        let key = format!("csrf:{}", token);

        let script = redis::Script::new(
            r"
            if redis.call('exists', KEYS[1]) == 1 then
                redis.call('del', KEYS[1])
                return 1
            else
                return 0
            end
            ",
        );

        let result: i32 = script.key(&key).invoke_async(&mut self.client).await?;
        Ok(result == 1)
    }
}

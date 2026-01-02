//! Medium HTTP client using the Typestate pattern
//!
//! The client must be initialized before making requests.
//! Invalid states (like fetching without initialization) are compile-time errors.

use crate::error::{Error, ErrorContext, Result};
use crate::parser::ArticleParser;
use crate::types::{Article, ArticleId};
use std::marker::PhantomData;
use std::time::Duration;

/// Marker type: Client is not yet initialized
pub struct Uninitialized;

/// Marker type: Client is ready to make requests
pub struct Ready;

/// Medium HTTP client with typestate pattern
///
/// The client transitions from `Uninitialized` to `Ready` after calling `init()`.
/// You cannot call `fetch_article` on an uninitialized client - it won't compile.
pub struct MediumClient<State> {
    client: Option<reqwest::Client>,
    user_agent: String,
    _state: PhantomData<State>,
}

impl MediumClient<Uninitialized> {
    /// Create a new uninitialized client
    pub fn new() -> Self {
        Self {
            client: None,
            user_agent: "Mozilla/5.0 (compatible; MediumMCP/1.0)".to_string(),
            _state: PhantomData,
        }
    }

    /// Set a custom user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Initialize the client, transitioning to Ready state
    ///
    /// This consumes the Uninitialized client and returns a Ready client.
    /// After this, you can make HTTP requests.
    pub fn init(self) -> Result<MediumClient<Ready>> {
        let client = reqwest::Client::builder()
            .user_agent(&self.user_agent)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Fetch {
                url: "client initialization".to_string(),
                source: e,
            })?;

        Ok(MediumClient {
            client: Some(client),
            user_agent: self.user_agent,
            _state: PhantomData,
        })
    }
}

impl Default for MediumClient<Uninitialized> {
    fn default() -> Self {
        Self::new()
    }
}

impl MediumClient<Ready> {
    /// Fetch an article by URL
    pub async fn fetch_article(&self, url: &str) -> Result<Article> {
        let client = self.client.as_ref().expect("Client must be initialized");

        let response = client.get(url).send().await.with_context(url)?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(Error::RateLimited { retry_after });
        }

        // Check for 404
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            let article_id = ArticleId::from_url(url)
                .map(|id| id.to_string())
                .unwrap_or_else(|| url.to_string());
            return Err(Error::NotFound { article_id });
        }

        // Check for other errors
        let response = response.error_for_status().with_context(url)?;

        let html = response.text().await.with_context(url)?;

        ArticleParser::parse(&html, url)
    }

    /// Fetch an article by ID (constructs the URL)
    pub async fn fetch_by_id(&self, article_id: &ArticleId, author: &str) -> Result<Article> {
        let url = format!("https://medium.com/@{}/{}", author, article_id);
        self.fetch_article(&url).await
    }

    /// Search for articles by query (returns URLs)
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let client = self.client.as_ref().expect("Client must be initialized");
        let search_url = format!("https://medium.com/search?q={}", urlencoding::encode(query));

        let response = client
            .get(&search_url)
            .send()
            .await
            .with_context(&search_url)?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::RateLimited { retry_after: None });
        }

        let html = response.text().await.with_context(&search_url)?;

        // Extract article URLs from search results
        let urls = ArticleParser::extract_article_urls(&html, limit);
        Ok(urls)
    }
}

// Ensure the client can't be cloned in an uninitialized state
// but can be cloned when ready (if needed)
impl Clone for MediumClient<Ready> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            user_agent: self.user_agent.clone(),
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_prevents_invalid_usage() {
        // This compiles
        let client = MediumClient::new();
        let _ready = client.init();

        // This would NOT compile (fetch_article not available on Uninitialized):
        // let client = MediumClient::new();
        // client.fetch_article("..."); // Error: method not found
    }

    #[test]
    fn test_custom_user_agent() {
        let client = MediumClient::new()
            .with_user_agent("CustomAgent/1.0")
            .init();
        assert!(client.is_ok());
    }
}

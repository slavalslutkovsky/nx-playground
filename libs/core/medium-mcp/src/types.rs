//! Domain types using the Newtype pattern
//!
//! Wrapping primitive types to:
//! - Prevent accidental confusion between IDs
//! - Add domain-specific validation
//! - Zero-cost abstraction at runtime

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Deref;

/// Article ID - wraps the Medium article slug/hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArticleId(String);

impl ArticleId {
    /// Create a new ArticleId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Extract article ID from a Medium URL
    pub fn from_url(url: &str) -> Option<Self> {
        // Medium URLs: https://medium.com/@user/article-title-abc123def456
        // or: https://user.medium.com/article-title-abc123def456
        url.split('/')
            .next_back()
            .map(|s| s.split('-').next_back().unwrap_or(s))
            .filter(|s| !s.is_empty())
            .map(|s| Self(s.to_string()))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn asd() {
  let ad = "titk".to_string();
  Article::new(ArticleId::new("123"),
               "Title",
               Author::new(AuthorId::new("test"), "Test"), 
               "Content", 
               "https://medium.com/test"
  );
}
impl Deref for ArticleId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Author ID - wraps the Medium username
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorId(String);

impl AuthorId {
    /// Create a new AuthorId from a username
    pub fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        // Strip @ if present
        let clean = username.strip_prefix('@').unwrap_or(&username);
        Self(clean.to_string())
    }

    /// Get the username with @ prefix
    pub fn with_at(&self) -> String {
        format!("@{}", self.0)
    }
}

impl Deref for AuthorId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// Tag/category for articles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into().to_lowercase())
    }
}

impl Deref for Tag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: AuthorId,
    pub name: String,
    pub bio: Option<String>,
    pub followers: Option<u64>,
}

impl Author {
    pub fn new(id: AuthorId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            bio: None,
            followers: None,
        }
    }

    pub fn with_bio(mut self, bio: impl Into<String>) -> Self {
        self.bio = Some(bio.into());
        self
    }

    pub fn with_followers(mut self, count: u64) -> Self {
        self.followers = Some(count);
        self
    }
}

/// Parsed Medium article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: ArticleId,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Author,
    pub content: String,
    pub tags: Vec<Tag>,
    pub claps: Option<u64>,
    pub read_time_minutes: Option<u32>,
    pub published_at: Option<String>,
    pub url: String,
}

impl Article {
    /// Create a new article with required fields
    pub fn new(
        id: ArticleId,
        title: impl Into<String>,
        author: Author,
        content: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            author,
            content: content.into(),
            tags: Vec::new(),
            claps: None,
            read_time_minutes: None,
            published_at: None,
            url: url.into(),
        }
    }

    /// Builder method for subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Builder method for tags
    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder method for claps
    pub fn with_claps(mut self, claps: u64) -> Self {
        self.claps = Some(claps);
        self
    }

    /// Builder method for read time
    pub fn with_read_time(mut self, minutes: u32) -> Self {
        self.read_time_minutes = Some(minutes);
        self
    }

    /// Get a summary of the article using Cow for efficiency
    /// Only allocates if content needs truncation
    pub fn summary(&self, max_chars: usize) -> Cow<'_, str> {
        if self.content.len() <= max_chars {
            Cow::Borrowed(&self.content)
        } else {
            let truncated: String = self.content.chars().take(max_chars).collect();
            Cow::Owned(format!("{}...", truncated))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_id_from_url() {
        let url = "https://medium.com/@user/my-article-title-abc123def";
        let id = ArticleId::from_url(url);
        assert!(id.is_some());
    }

    #[test]
    fn test_author_id_strips_at() {
        let author = AuthorId::new("@username");
        assert_eq!(&*author, "username");
        assert_eq!(author.with_at(), "@username");
    }

    #[test]
    fn test_cow_summary_no_alloc() {
        let author = Author::new(AuthorId::new("test"), "Test");
        let article = Article::new(
            ArticleId::new("123"),
            "Title",
            author,
            "Short content",
            "https://medium.com/test",
        );

        let summary = article.summary(100);
        assert!(matches!(summary, Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_summary_truncates() {
        let author = Author::new(AuthorId::new("test"), "Test");
        let article = Article::new(
            ArticleId::new("123"),
            "Title",
            author,
            "This is a long content that should be truncated",
            "https://medium.com/test",
        );

        let summary = article.summary(10);
        assert!(matches!(summary, Cow::Owned(_)));
        assert!(summary.ends_with("..."));
    }
}

//! HTML parser for Medium articles
//!
//! Uses the scraper crate to extract article content from Medium's HTML.

use crate::error::{Error, Result};
use crate::types::{Article, ArticleId, Author, AuthorId, Tag};
use scraper::{Html, Selector};

/// Parser for Medium article HTML
pub struct ArticleParser;

impl ArticleParser {
    /// Parse an article from HTML content
    pub fn parse(html: &str, url: &str) -> Result<Article> {
        let document = Html::parse_document(html);

        // Extract article ID from URL
        let article_id = ArticleId::from_url(url).ok_or_else(|| Error::Parse {
            context: "article_id".to_string(),
            details: "Could not extract article ID from URL".to_string(),
        })?;

        // Extract title
        let title = Self::extract_title(&document)?;

        // Extract author
        let author = Self::extract_author(&document)?;

        // Extract content
        let content = Self::extract_content(&document)?;

        // Extract optional fields
        let subtitle = Self::extract_subtitle(&document);
        let tags = Self::extract_tags(&document);
        let read_time = Self::extract_read_time(&document);
        let claps = Self::extract_claps(&document);

        let mut article = Article::new(article_id, title, author, content, url);

        if let Some(subtitle) = subtitle {
            article = article.with_subtitle(subtitle);
        }

        if !tags.is_empty() {
            article = article.with_tags(tags);
        }

        if let Some(minutes) = read_time {
            article = article.with_read_time(minutes);
        }

        if let Some(claps) = claps {
            article = article.with_claps(claps);
        }

        Ok(article)
    }

    /// Extract article title
    fn extract_title(document: &Html) -> Result<String> {
        // Try multiple selectors for title
        let selectors = [
            "h1[data-testid='storyTitle']",
            "article h1",
            "h1.pw-post-title",
            "h1",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let title: String = element.text().collect();
                let title = title.trim();
                if !title.is_empty() {
                    return Ok(title.to_string());
                }
            }
        }

        // Fallback to meta title
        if let Ok(selector) = Selector::parse("meta[property='og:title']")
            && let Some(element) = document.select(&selector).next()
            && let Some(title) = element.value().attr("content")
        {
            return Ok(title.to_string());
        }

        Err(Error::Parse {
            context: "title".to_string(),
            details: "Could not find article title".to_string(),
        })
    }

    /// Extract author information
    fn extract_author(document: &Html) -> Result<Author> {
        // Try to find author link
        let author_selectors = [
            "a[data-testid='authorName']",
            "a[rel='author']",
            ".pw-author-name",
        ];

        for selector_str in author_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let name: String = element.text().collect();
                let name = name.trim();

                // Try to get username from href
                let username = element
                    .value()
                    .attr("href")
                    .and_then(|href| {
                        href.split('@')
                            .nth(1)
                            .map(|s| s.split('/').next().unwrap_or(s))
                    })
                    .unwrap_or(name);

                if !name.is_empty() {
                    return Ok(Author::new(AuthorId::new(username), name));
                }
            }
        }

        // Fallback to meta author
        if let Ok(selector) = Selector::parse("meta[name='author']")
            && let Some(element) = document.select(&selector).next()
            && let Some(name) = element.value().attr("content")
        {
            return Ok(Author::new(AuthorId::new(name), name));
        }

        Err(Error::Parse {
            context: "author".to_string(),
            details: "Could not find author information".to_string(),
        })
    }

    /// Extract article content
    fn extract_content(document: &Html) -> Result<String> {
        // Try to find article content
        let content_selectors = [
            "article section",
            "article .pw-post-body-paragraph",
            "article p",
        ];

        for selector_str in content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let paragraphs: Vec<String> = document
                    .select(&selector)
                    .map(|el| {
                        let text: String = el.text().collect();
                        text.trim().to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                if !paragraphs.is_empty() {
                    return Ok(paragraphs.join("\n\n"));
                }
            }
        }

        // Fallback to meta description
        if let Ok(selector) = Selector::parse("meta[property='og:description']")
            && let Some(element) = document.select(&selector).next()
            && let Some(content) = element.value().attr("content")
        {
            return Ok(content.to_string());
        }

        Err(Error::Parse {
            context: "content".to_string(),
            details: "Could not extract article content".to_string(),
        })
    }

    /// Extract subtitle (optional)
    fn extract_subtitle(document: &Html) -> Option<String> {
        let selectors = [
            "h2[data-testid='storySubtitle']",
            "article h2:first-of-type",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let subtitle: String = element.text().collect();
                let subtitle = subtitle.trim();
                if !subtitle.is_empty() {
                    return Some(subtitle.to_string());
                }
            }
        }

        None
    }

    /// Extract tags
    fn extract_tags(document: &Html) -> Vec<Tag> {
        if let Ok(selector) = Selector::parse("a[href*='/tag/']") {
            return document
                .select(&selector)
                .filter_map(|el| {
                    let text: String = el.text().collect();
                    let text = text.trim();
                    if !text.is_empty() {
                        Some(Tag::new(text))
                    } else {
                        None
                    }
                })
                .collect();
        }

        Vec::new()
    }

    /// Extract read time in minutes
    fn extract_read_time(document: &Html) -> Option<u32> {
        if let Ok(selector) = Selector::parse("[data-testid='storyReadTime']")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            // Parse "X min read" format
            return text.split_whitespace().next().and_then(|s| s.parse().ok());
        }

        None
    }

    /// Extract clap count
    fn extract_claps(document: &Html) -> Option<u64> {
        if let Ok(selector) = Selector::parse("button[data-testid='headerClapButton'] span")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            // Parse "1.2K" format
            return Self::parse_count(&text);
        }

        None
    }

    /// Parse count with K/M suffix
    fn parse_count(text: &str) -> Option<u64> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        let multiplier = if text.ends_with('K') || text.ends_with('k') {
            1000
        } else if text.ends_with('M') || text.ends_with('m') {
            1_000_000
        } else {
            1
        };

        let number_str = text.trim_end_matches(['K', 'k', 'M', 'm']);
        number_str
            .parse::<f64>()
            .ok()
            .map(|n| (n * multiplier as f64) as u64)
    }

    /// Extract article URLs from search results page
    pub fn extract_article_urls(html: &str, limit: usize) -> Vec<String> {
        let document = Html::parse_document(html);

        if let Ok(selector) = Selector::parse("a[href*='medium.com']") {
            return document
                .select(&selector)
                .filter_map(|el| el.value().attr("href"))
                .filter(|href| {
                    // Filter to actual article URLs (contain hash at end)
                    href.contains("medium.com")
                        && !href.contains("/tag/")
                        && !href.contains("/search")
                        && href.split('-').count() > 2
                })
                .take(limit)
                .map(String::from)
                .collect();
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_count() {
        assert_eq!(ArticleParser::parse_count("100"), Some(100));
        assert_eq!(ArticleParser::parse_count("1.2K"), Some(1200));
        assert_eq!(ArticleParser::parse_count("1.5M"), Some(1_500_000));
        assert_eq!(ArticleParser::parse_count(""), None);
    }
}

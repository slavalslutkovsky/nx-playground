//! MCP (Model Context Protocol) handler
//!
//! Implements a JSON-RPC style MCP server for AI model integration.
//! Uses sealed traits to control which tools can be registered.

use crate::Ready;
use crate::client::MediumClient;
use crate::error::{Error, ErrorContext, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Sealed Trait Pattern - Prevents external implementations
// ============================================================================

mod sealed {
    pub trait Sealed {}
}

/// Trait for MCP tools - sealed to prevent external implementations
#[async_trait]
pub trait Tool: sealed::Sealed + Send + Sync {
    /// Tool name
    fn name(&self) -> &'static str;

    /// Tool description for the AI model
    fn description(&self) -> &'static str;

    /// JSON schema for input parameters
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// MCP request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// MCP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl McpResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

// ============================================================================
// Built-in Tools
// ============================================================================

/// Tool to fetch a Medium article
pub struct FetchArticleTool {
    client: Arc<MediumClient<Ready>>,
}

impl sealed::Sealed for FetchArticleTool {}

#[async_trait]
impl Tool for FetchArticleTool {
    fn name(&self) -> &'static str {
        "fetch_medium_article"
    }

    fn description(&self) -> &'static str {
        "Fetches and parses a Medium article given its URL. Returns the article title, author, content, tags, and metadata."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The full Medium article URL"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let url =
            params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidRequest {
                    message: "Missing 'url' parameter".to_string(),
                })?;

        let article = self.client.fetch_article(url).await?;
        serde_json::to_value(article).with_context("article serialization")
    }
}

/// Tool to search Medium articles
pub struct SearchArticlesTool {
    client: Arc<MediumClient<Ready>>,
}

impl sealed::Sealed for SearchArticlesTool {}

#[async_trait]
impl Tool for SearchArticlesTool {
    fn name(&self) -> &'static str {
        "search_medium_articles"
    }

    fn description(&self) -> &'static str {
        "Searches Medium for articles matching a query. Returns a list of article URLs."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 10)",
                    "default": 10
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest {
                message: "Missing 'query' parameter".to_string(),
            })?;

        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let urls = self.client.search(query, limit).await?;
        serde_json::to_value(urls).with_context("search results serialization")
    }
}

/// Tool to extract article summary
pub struct SummarizeArticleTool {
    client: Arc<MediumClient<Ready>>,
}

impl sealed::Sealed for SummarizeArticleTool {}

#[async_trait]
impl Tool for SummarizeArticleTool {
    fn name(&self) -> &'static str {
        "summarize_medium_article"
    }

    fn description(&self) -> &'static str {
        "Fetches a Medium article and returns a summary with key information."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The full Medium article URL"
                },
                "max_length": {
                    "type": "integer",
                    "description": "Maximum summary length in characters (default: 500)",
                    "default": 500
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let url =
            params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidRequest {
                    message: "Missing 'url' parameter".to_string(),
                })?;

        let max_length = params
            .get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(500) as usize;

        let article = self.client.fetch_article(url).await?;

        let summary = serde_json::json!({
            "title": article.title,
            "subtitle": article.subtitle,
            "author": article.author.name,
            "summary": article.summary(max_length),
            "tags": article.tags,
            "read_time_minutes": article.read_time_minutes,
            "claps": article.claps
        });

        Ok(summary)
    }
}

// ============================================================================
// MCP Handler
// ============================================================================

/// MCP request handler
pub struct McpHandler {
    tools: HashMap<String, Box<dyn Tool>>,
    server_info: ServerInfo,
}

#[derive(Clone, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

impl McpHandler {
    /// Create a new MCP handler with the Medium client
    pub fn new(client: MediumClient<Ready>) -> Self {
        let client = Arc::new(client);
        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();

        // Register built-in tools
        let fetch_tool = FetchArticleTool {
            client: Arc::clone(&client),
        };
        tools.insert(fetch_tool.name().to_string(), Box::new(fetch_tool));

        let search_tool = SearchArticlesTool {
            client: Arc::clone(&client),
        };
        tools.insert(search_tool.name().to_string(), Box::new(search_tool));

        let summarize_tool = SummarizeArticleTool { client };
        tools.insert(summarize_tool.name().to_string(), Box::new(summarize_tool));

        Self {
            tools,
            server_info: ServerInfo {
                name: "medium-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Handle an MCP request
    pub async fn handle(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            _ => McpResponse::error(request.id, -32601, "Method not found"),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&self, id: Option<serde_json::Value>) -> McpResponse {
        McpResponse::success(
            id,
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": self.server_info,
                "capabilities": {
                    "tools": {}
                }
            }),
        )
    }

    /// Handle tools/list request
    fn handle_list_tools(&self, id: Option<serde_json::Value>) -> McpResponse {
        let tools: Vec<serde_json::Value> = self
            .tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.input_schema()
                })
            })
            .collect();

        McpResponse::success(id, serde_json::json!({ "tools": tools }))
    }

    /// Handle tools/call request
    async fn handle_call_tool(
        &self,
        id: Option<serde_json::Value>,
        params: serde_json::Value,
    ) -> McpResponse {
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return McpResponse::error(id, -32602, "Missing tool name"),
        };

        let tool = match self.tools.get(tool_name) {
            Some(tool) => tool,
            None => return McpResponse::error(id, -32602, format!("Unknown tool: {}", tool_name)),
        };

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match tool.execute(arguments).await {
            Ok(result) => McpResponse::success(
                id,
                serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                    }]
                }),
            ),
            Err(e) => McpResponse::error(id, -32000, e.to_string()),
        }
    }

    /// Parse and handle a JSON request string
    pub async fn handle_json(&self, json: &str) -> String {
        let request: McpRequest = match serde_json::from_str(json) {
            Ok(req) => req,
            Err(e) => {
                let response = McpResponse::error(None, -32700, format!("Parse error: {}", e));
                return serde_json::to_string(&response).unwrap_or_default();
            }
        };

        let response = self.handle(request).await;
        serde_json::to_string(&response).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_response_success() {
        let response =
            McpResponse::success(Some(serde_json::json!(1)), serde_json::json!({"ok": true}));
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_mcp_response_error() {
        let response = McpResponse::error(Some(serde_json::json!(1)), -32600, "Invalid request");
        assert!(response.error.is_some());
        assert!(response.result.is_none());
    }
}

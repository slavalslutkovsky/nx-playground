//! FinOps AI Agent Orchestrator
//!
//! The orchestrator coordinates the AI agent's interactions with the user,
//! routing requests to appropriate tools and managing conversation context.

use futures::stream::{self, Stream};
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use super::prompts::FINOPS_SYSTEM_PROMPT;
use super::tools::{
    ComparePricesTool, ExploreResourcesTool, GenerateRecommendationTool, ToolRegistry,
};
use crate::error::{FinopsError, FinopsResult};
use crate::models::{ChatChunk, ChatContext, ChatResponse, ToolCallRecord};
use crate::repository::FinopsRepository;
use crate::service::FinopsService;
use domain_pricing::PricingService;

/// FinOps AI Agent Orchestrator
///
/// Coordinates AI agent interactions, tool execution, and response generation.
pub struct FinopsOrchestrator<R: FinopsRepository> {
    service: Arc<FinopsService<R>>,
    tools: Arc<ToolRegistry>,
    openai_api_key: String,
}

impl<R: FinopsRepository + 'static> FinopsOrchestrator<R> {
    /// Create a new orchestrator
    pub fn new(service: FinopsService<R>) -> Self {
        let openai_api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test".to_string());

        Self {
            service: Arc::new(service),
            tools: Arc::new(ToolRegistry::new()),
            openai_api_key,
        }
    }

    /// Create orchestrator with pricing service for price comparison tools
    pub fn with_pricing_service<P: domain_pricing::repository::PricingRepository + 'static>(
        service: FinopsService<R>,
        pricing_service: Arc<PricingService<P>>,
    ) -> Self {
        let openai_api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test".to_string());

        let service = Arc::new(service);

        // Build tool registry
        let mut tools = ToolRegistry::new();
        tools.register(ComparePricesTool::new(pricing_service));
        tools.register(ExploreResourcesTool::new(service.clone()));
        tools.register(GenerateRecommendationTool::new(service.clone()));

        Self {
            service,
            tools: Arc::new(tools),
            openai_api_key,
        }
    }

    /// Handle a chat message and return a response
    pub async fn chat(
        &self,
        session_id: Uuid,
        message: &str,
        context: &ChatContext,
    ) -> FinopsResult<ChatResponse> {
        // Build conversation context
        let conversation = self
            .service
            .build_conversation_context(session_id)
            .await
            .unwrap_or_default();

        // Build the full prompt with context
        let full_prompt = self.build_prompt(message, context, &conversation);

        // Call OpenAI API
        let response = self.call_openai(&full_prompt).await?;

        // Parse tool calls if any
        let (content, tool_calls) = self.process_response(&response).await?;

        Ok(ChatResponse {
            session_id,
            content,
            tool_calls,
        })
    }

    /// Stream chat response chunks
    pub async fn chat_stream(
        &self,
        session_id: Uuid,
        message: &str,
        context: &ChatContext,
    ) -> FinopsResult<Pin<Box<dyn Stream<Item = FinopsResult<ChatChunk>> + Send>>> {
        // Build conversation context
        let conversation = self
            .service
            .build_conversation_context(session_id)
            .await
            .unwrap_or_default();

        // Build the full prompt with context
        let full_prompt = self.build_prompt(message, context, &conversation);

        // For now, we'll simulate streaming by returning the full response in chunks
        // In a real implementation, this would use OpenAI's streaming API
        let response = self.call_openai(&full_prompt).await?;
        let (content, _tool_calls) = self.process_response(&response).await?;

        // Create a stream of chunks
        let session_id_clone = session_id;
        let chunks: Vec<FinopsResult<ChatChunk>> = vec![
            Ok(ChatChunk::Text { content }),
            Ok(ChatChunk::Done {
                session_id: session_id_clone,
            }),
        ];

        Ok(Box::pin(stream::iter(chunks)))
    }

    /// Build the full prompt with system instructions and context
    fn build_prompt(&self, message: &str, context: &ChatContext, conversation: &str) -> String {
        let mut prompt = String::new();

        // System prompt
        prompt.push_str(FINOPS_SYSTEM_PROMPT);
        prompt.push_str("\n\n");

        // User context
        prompt.push_str("## User Context\n\n");

        if !context.preferred_providers.is_empty() {
            prompt.push_str(&format!(
                "- Preferred providers: {:?}\n",
                context.preferred_providers
            ));
        }

        if let Some(budget) = context.budget_monthly {
            prompt.push_str(&format!(
                "- Monthly budget: ${:.2}\n",
                budget as f64 / 100.0
            ));
        }

        if !context.regions.is_empty() {
            prompt.push_str(&format!("- Preferred regions: {:?}\n", context.regions));
        }

        if !context.compliance_requirements.is_empty() {
            prompt.push_str(&format!(
                "- Compliance requirements: {:?}\n",
                context.compliance_requirements
            ));
        }

        prompt.push('\n');

        // Available tools
        prompt.push_str("## Available Tools\n\n");
        for (name, description, _schema) in self.tools.list() {
            prompt.push_str(&format!("- `{}`: {}\n", name, description));
        }
        prompt.push('\n');

        // Conversation history
        if !conversation.is_empty() {
            prompt.push_str("## Conversation History\n\n");
            prompt.push_str(conversation);
            prompt.push_str("\n\n");
        }

        // Current message
        prompt.push_str("## Current Request\n\n");
        prompt.push_str(&format!("User: {}\n", message));

        prompt
    }

    /// Call OpenAI API using reqwest
    async fn call_openai(&self, prompt: &str) -> FinopsResult<String> {
        let client = reqwest::Client::new();

        let request_body = serde_json::json!({
            "model": "gpt-4o",
            "messages": [
                {
                    "role": "system",
                    "content": FINOPS_SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4096
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| FinopsError::Agent(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FinopsError::Agent(format!(
                "OpenAI API error {}: {}",
                status, body
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| FinopsError::Agent(format!("Failed to parse response: {}", e)))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    /// Process the response, extracting tool calls and executing them
    async fn process_response(
        &self,
        response: &str,
    ) -> FinopsResult<(String, Vec<ToolCallRecord>)> {
        // Check if the response contains tool call markers
        // This is a simple heuristic - in production you'd use proper function calling
        if response.contains("```tool:") {
            let mut content = String::new();
            let mut tool_calls = Vec::new();

            for part in response.split("```") {
                if part.starts_with("tool:") {
                    // Parse and execute tool call
                    let tool_result = self.execute_tool_from_response(part).await;
                    match tool_result {
                        Ok((name, args, result)) => {
                            tool_calls.push(ToolCallRecord {
                                name,
                                arguments: args,
                                result: Some(result.clone()),
                                latency_ms: None,
                            });
                            content.push_str(&result);
                            content.push_str("\n\n");
                        }
                        Err(e) => {
                            content.push_str(&format!("Tool error: {}\n\n", e));
                        }
                    }
                } else {
                    content.push_str(part.trim());
                    content.push('\n');
                }
            }

            Ok((content.trim().to_string(), tool_calls))
        } else {
            Ok((response.to_string(), Vec::new()))
        }
    }

    /// Execute a tool from a response marker
    async fn execute_tool_from_response(
        &self,
        tool_marker: &str,
    ) -> FinopsResult<(String, String, String)> {
        // Parse: "tool:tool_name\n{json args}"
        let lines: Vec<&str> = tool_marker.lines().collect();
        if lines.is_empty() {
            return Err(FinopsError::ToolExecution("Empty tool marker".to_string()));
        }

        let tool_name = lines[0].trim_start_matches("tool:").trim();
        let args_json = lines[1..].join("\n");

        let arguments: serde_json::Value = serde_json::from_str(&args_json)
            .map_err(|e| FinopsError::ToolExecution(format!("Invalid JSON: {}", e)))?;

        let result = self.tools.execute(tool_name, arguments.clone()).await?;

        Ok((
            tool_name.to_string(),
            serde_json::to_string(&arguments).unwrap_or_default(),
            result,
        ))
    }
}

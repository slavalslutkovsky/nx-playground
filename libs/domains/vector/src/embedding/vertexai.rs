//! Vertex AI embedding provider implementation
//!
//! Uses Google Cloud's Vertex AI text embedding API.
//! Supports authentication via:
//! - Service account JSON file (GOOGLE_APPLICATION_CREDENTIALS)
//! - Workload Identity (in GKE)
//! - Default application credentials

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::EmbeddingProvider;
use crate::error::{VectorError, VectorResult};
use crate::models::{EmbeddingModel, EmbeddingProviderType, EmbeddingResult};

/// Vertex AI provider configuration
#[derive(Debug, Clone)]
pub struct VertexAIConfig {
    /// GCP Project ID
    pub project_id: String,
    /// GCP Region (e.g., "us-central1")
    pub location: String,
    /// Access token (obtained from Google Auth)
    /// If not provided, will attempt to use Application Default Credentials
    pub access_token: Option<String>,
}

impl VertexAIConfig {
    pub fn new(project_id: String, location: String) -> Self {
        Self {
            project_id,
            location,
            access_token: None,
        }
    }

    pub fn with_access_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }

    pub fn from_env() -> VectorResult<Self> {
        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| std::env::var("GCP_PROJECT_ID"))
            .map_err(|_| {
                VectorError::Config("GOOGLE_CLOUD_PROJECT or GCP_PROJECT_ID not set".to_string())
            })?;

        let location =
            std::env::var("VERTEX_AI_LOCATION").unwrap_or_else(|_| "us-central1".to_string());

        let access_token = std::env::var("GOOGLE_ACCESS_TOKEN").ok();

        Ok(Self {
            project_id,
            location,
            access_token,
        })
    }

    /// Get the Vertex AI endpoint URL for the given model
    fn endpoint_url(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            self.location, self.project_id, self.location, model
        )
    }
}

/// Vertex AI embeddings provider
pub struct VertexAIProvider {
    client: Client,
    config: VertexAIConfig,
}

impl VertexAIProvider {
    pub fn new(config: VertexAIConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn from_env() -> VectorResult<Self> {
        Ok(Self::new(VertexAIConfig::from_env()?))
    }

    /// Get access token, refreshing if needed
    async fn get_access_token(&self) -> VectorResult<String> {
        // If we have a configured token, use it
        if let Some(ref token) = self.config.access_token {
            return Ok(token.clone());
        }

        // Try to get token from metadata server (for GKE workload identity)
        self.get_metadata_token().await
    }

    /// Get access token from GCP metadata server (works in GKE with Workload Identity)
    async fn get_metadata_token(&self) -> VectorResult<String> {
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

        let response = self
            .client
            .get(metadata_url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|e| {
                VectorError::Config(format!(
                    "Failed to get access token from metadata server: {}. \
                     Set GOOGLE_ACCESS_TOKEN environment variable for local development.",
                    e
                ))
            })?;

        if !response.status().is_success() {
            return Err(VectorError::Config(
                "Failed to get access token from metadata server. \
                 Set GOOGLE_ACCESS_TOKEN environment variable for local development."
                    .to_string(),
            ));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| VectorError::Config(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response.access_token)
    }

    /// Map EmbeddingModel to Vertex AI model name
    fn model_name(model: EmbeddingModel) -> &'static str {
        match model {
            EmbeddingModel::Gecko => "textembedding-gecko@003",
            EmbeddingModel::GeckoMultilingual => "textembedding-gecko-multilingual@001",
            EmbeddingModel::TextEmbedding004 => "text-embedding-004",
            EmbeddingModel::TextEmbedding005 => "text-embedding-005",
            EmbeddingModel::TextMultilingualEmbedding002 => "text-multilingual-embedding-002",
            // Default to text-embedding-004 for non-Vertex models
            _ => "text-embedding-004",
        }
    }
}

// Vertex AI request/response types

#[derive(Debug, Serialize)]
struct VertexAIRequest {
    instances: Vec<TextInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<EmbeddingParameters>,
}

#[derive(Debug, Serialize)]
struct TextInstance {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmbeddingParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimensionality: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct VertexAIResponse {
    predictions: Vec<EmbeddingPrediction>,
    #[serde(default)]
    metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingPrediction {
    embeddings: EmbeddingValues,
}

#[derive(Debug, Deserialize)]
struct EmbeddingValues {
    values: Vec<f32>,
    #[serde(default)]
    statistics: Option<EmbeddingStatistics>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingStatistics {
    #[serde(default)]
    token_count: u32,
}

#[derive(Debug, Deserialize)]
struct ResponseMetadata {
    #[serde(default, rename = "billableCharacterCount")]
    billable_character_count: u64,
}

#[async_trait]
impl EmbeddingProvider for VertexAIProvider {
    fn provider_type(&self) -> EmbeddingProviderType {
        EmbeddingProviderType::VertexAI
    }

    async fn embed(&self, model: EmbeddingModel, text: &str) -> VectorResult<EmbeddingResult> {
        let results = self.embed_batch(model, &[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| VectorError::Embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(
        &self,
        model: EmbeddingModel,
        texts: &[String],
    ) -> VectorResult<Vec<EmbeddingResult>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let access_token = self.get_access_token().await?;
        let model_name = Self::model_name(model);
        let endpoint = self.config.endpoint_url(model_name);

        // Create instances for each text
        let instances: Vec<TextInstance> = texts
            .iter()
            .map(|text| TextInstance {
                content: text.clone(),
                task_type: Some("RETRIEVAL_DOCUMENT".to_string()),
                title: None,
            })
            .collect();

        let request = VertexAIRequest {
            instances,
            parameters: None,
        };

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VectorError::Embedding(format!(
                "Vertex AI API error ({}): {}",
                status, error_text
            )));
        }

        let embedding_response: VertexAIResponse = response.json().await?;

        Ok(embedding_response
            .predictions
            .into_iter()
            .map(|p| {
                let values = p.embeddings.values;
                let dimension = values.len() as u32;
                let tokens_used = p.embeddings.statistics.map(|s| s.token_count).unwrap_or(0);

                EmbeddingResult {
                    values,
                    dimension,
                    tokens_used,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_names() {
        assert_eq!(
            VertexAIProvider::model_name(EmbeddingModel::Gecko),
            "textembedding-gecko@003"
        );
        assert_eq!(
            VertexAIProvider::model_name(EmbeddingModel::TextEmbedding004),
            "text-embedding-004"
        );
        assert_eq!(
            VertexAIProvider::model_name(EmbeddingModel::TextEmbedding005),
            "text-embedding-005"
        );
    }

    #[test]
    fn test_endpoint_url() {
        let config = VertexAIConfig::new("my-project".to_string(), "us-central1".to_string());
        let expected = "https://us-central1-aiplatform.googleapis.com/v1/projects/my-project/locations/us-central1/publishers/google/models/text-embedding-004:predict";
        assert_eq!(config.endpoint_url("text-embedding-004"), expected);
    }
}

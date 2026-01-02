//! Dapr integration for event publishing and subscription

use crate::error::{EventError, Result};
use crate::models::{CloudEvent, Event};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

/// Dapr pub/sub client for event distribution
#[derive(Clone)]
pub struct DaprClient {
    client: reqwest::Client,
    dapr_http_port: u16,
    pubsub_name: String,
}

impl DaprClient {
    /// Create a new Dapr client
    pub fn new(dapr_http_port: u16, pubsub_name: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            dapr_http_port,
            pubsub_name: pubsub_name.into(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        let port = std::env::var("DAPR_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3500);

        let pubsub_name =
            std::env::var("DAPR_PUBSUB_NAME").unwrap_or_else(|_| "events-pubsub".to_string());

        Self::new(port, pubsub_name)
    }

    /// Get the Dapr base URL
    fn base_url(&self) -> String {
        format!("http://localhost:{}", self.dapr_http_port)
    }

    /// Publish an event to a topic
    #[instrument(skip(self, data), fields(topic = %topic))]
    pub async fn publish<T: Serialize>(&self, topic: &str, data: &T) -> Result<()> {
        let url = format!(
            "{}/v1.0/publish/{}/{}",
            self.base_url(),
            self.pubsub_name,
            topic
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await
            .map_err(|e| EventError::Dapr {
                message: format!("Failed to publish to {}: {}", topic, e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Dapr publish failed");
            return Err(EventError::Dapr {
                message: format!("Publish failed with status {}: {}", status, body),
            });
        }

        info!(topic = %topic, "Event published to Dapr");
        Ok(())
    }

    /// Publish with CloudEvent envelope
    #[instrument(skip(self, data), fields(topic = %topic, event_type = %event_type))]
    pub async fn publish_cloud_event<T: Serialize>(
        &self,
        topic: &str,
        event_type: &str,
        source: &str,
        data: T,
    ) -> Result<()> {
        let cloud_event = CloudEvent::new(event_type, source, data);
        self.publish(topic, &cloud_event).await
    }

    /// Invoke another service via Dapr service invocation
    #[instrument(skip(self, body))]
    pub async fn invoke<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        app_id: &str,
        method: &str,
        body: Option<&T>,
    ) -> Result<R> {
        let url = format!(
            "{}/v1.0/invoke/{}/method/{}",
            self.base_url(),
            app_id,
            method
        );

        let mut request = self.client.post(&url);

        if let Some(data) = body {
            request = request.json(data);
        }

        let response = request.send().await.map_err(|e| EventError::Dapr {
            message: format!("Failed to invoke {}/{}: {}", app_id, method, e),
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventError::Dapr {
                message: format!("Invoke failed with status {}: {}", status, body),
            });
        }

        response.json().await.map_err(|e| EventError::Dapr {
            message: format!("Failed to parse response: {}", e),
        })
    }

    /// Save state to Dapr state store
    #[instrument(skip(self, value))]
    pub async fn save_state<T: Serialize>(
        &self,
        store_name: &str,
        key: &str,
        value: &T,
    ) -> Result<()> {
        let url = format!("{}/v1.0/state/{}", self.base_url(), store_name);

        let state_item = serde_json::json!([{
            "key": key,
            "value": value
        }]);

        let response = self
            .client
            .post(&url)
            .json(&state_item)
            .send()
            .await
            .map_err(|e| EventError::Dapr {
                message: format!("Failed to save state: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventError::Dapr {
                message: format!("Save state failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    /// Get state from Dapr state store
    #[instrument(skip(self))]
    pub async fn get_state<T: for<'de> Deserialize<'de>>(
        &self,
        store_name: &str,
        key: &str,
    ) -> Result<Option<T>> {
        let url = format!("{}/v1.0/state/{}/{}", self.base_url(), store_name, key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| EventError::Dapr {
                message: format!("Failed to get state: {}", e),
            })?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventError::Dapr {
                message: format!("Get state failed with status {}: {}", status, body),
            });
        }

        let value = response.json().await.map_err(|e| EventError::Dapr {
            message: format!("Failed to parse state: {}", e),
        })?;

        Ok(Some(value))
    }

    /// Output binding (write to InfluxDB via Dapr)
    #[instrument(skip(self, data))]
    pub async fn invoke_binding<T: Serialize>(
        &self,
        binding_name: &str,
        operation: &str,
        data: &T,
    ) -> Result<()> {
        let url = format!("{}/v1.0/bindings/{}", self.base_url(), binding_name);

        let payload = serde_json::json!({
            "operation": operation,
            "data": data
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| EventError::Dapr {
                message: format!("Failed to invoke binding {}: {}", binding_name, e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventError::Dapr {
                message: format!("Binding invocation failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    /// Health check for Dapr sidecar
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/v1.0/healthz", self.base_url());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| EventError::Dapr {
                message: format!("Health check failed: {}", e),
            })?;

        Ok(response.status().is_success())
    }
}

/// Event publisher that combines Dapr pub/sub with optional InfluxDB binding
pub struct DaprEventPublisher {
    /// The underlying Dapr client
    pub dapr: DaprClient,
    topic: String,
    source: String,
}

impl DaprEventPublisher {
    /// Create a new event publisher
    pub fn new(dapr: DaprClient, topic: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            dapr,
            topic: topic.into(),
            source: source.into(),
        }
    }

    /// Publish an event to the events topic
    #[instrument(skip(self, event), fields(event_id = %event.id, event_name = %event.name))]
    pub async fn publish(&self, event: &Event) -> Result<()> {
        // Publish as CloudEvent
        self.dapr
            .publish_cloud_event(
                &self.topic,
                &format!("event.{}", event.name),
                &self.source,
                event,
            )
            .await?;

        info!(event_id = %event.id, "Event published via Dapr");
        Ok(())
    }

    /// Publish event and write to InfluxDB via Dapr binding
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn publish_with_metrics(&self, event: &Event) -> Result<()> {
        // Publish to topic
        self.publish(event).await?;

        // Write metrics to InfluxDB via Dapr binding
        use crate::influxdb::EventMetrics;
        let metrics = EventMetrics::from_event(event);

        // Use line protocol for InfluxDB binding
        let line_protocol = metrics.to_line_protocol();

        self.dapr
            .invoke_binding("influxdb", "create", &line_protocol)
            .await
            .map_err(|e| {
                warn!(error = %e, "Failed to write to InfluxDB via Dapr binding");
                e
            })?;

        Ok(())
    }
}

/// Dapr subscription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaprSubscription {
    pub pubsubname: String,
    pub topic: String,
    pub route: String,
}

/// Dapr topic event wrapper (received from pub/sub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaprTopicEvent<T> {
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub specversion: String,
    pub datacontenttype: String,
    pub data: T,
    pub topic: String,
    pub pubsubname: String,
}

/// Dapr subscription response status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DaprSubscriptionStatus {
    Success,
    Retry,
    Drop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaprSubscriptionResponse {
    pub status: DaprSubscriptionStatus,
}

impl DaprSubscriptionResponse {
    pub fn success() -> Self {
        Self {
            status: DaprSubscriptionStatus::Success,
        }
    }

    pub fn retry() -> Self {
        Self {
            status: DaprSubscriptionStatus::Retry,
        }
    }

    pub fn drop() -> Self {
        Self {
            status: DaprSubscriptionStatus::Drop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dapr_client_creation() {
        let client = DaprClient::new(3500, "test-pubsub");
        assert_eq!(client.base_url(), "http://localhost:3500");
    }

    #[test]
    fn test_subscription_response() {
        let response = DaprSubscriptionResponse::success();
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("SUCCESS"));
    }
}

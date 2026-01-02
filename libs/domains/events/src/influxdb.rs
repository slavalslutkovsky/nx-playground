//! InfluxDB integration for event metrics and time-series storage

use crate::error::{EventError, Result};
use crate::models::{Event, EventCategory, EventSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, instrument, warn};

/// Event metrics point for InfluxDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    /// Measurement name
    pub measurement: String,

    /// Tags for indexing
    pub tags: HashMap<String, String>,

    /// Field values
    pub fields: HashMap<String, FieldValue>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Field value types for InfluxDB
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl EventMetrics {
    /// Create metrics from an event
    pub fn from_event(event: &Event) -> Self {
        let mut tags = HashMap::new();
        tags.insert("category".to_string(), event.category.to_string());
        tags.insert("severity".to_string(), event.severity.to_string());
        tags.insert("name".to_string(), event.name.clone());

        if let Some(source) = &event.metadata.source {
            tags.insert("source".to_string(), source.clone());
        }

        let mut fields = HashMap::new();
        fields.insert("count".to_string(), FieldValue::Integer(1));
        fields.insert(
            "message_length".to_string(),
            FieldValue::Integer(event.message.len() as i64),
        );

        // Add data size if present
        if !event.data.is_null() {
            if let Ok(json) = serde_json::to_string(&event.data) {
                fields.insert(
                    "data_size".to_string(),
                    FieldValue::Integer(json.len() as i64),
                );
            }
        }

        Self {
            measurement: "events".to_string(),
            tags,
            fields,
            timestamp: event.timestamp,
        }
    }

    /// Convert to InfluxDB line protocol
    pub fn to_line_protocol(&self) -> String {
        let tags: Vec<String> = self
            .tags
            .iter()
            .map(|(k, v)| format!("{}={}", escape_tag(k), escape_tag(v)))
            .collect();

        let fields: Vec<String> = self
            .fields
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    FieldValue::String(s) => format!("\"{}\"", escape_string(s)),
                    FieldValue::Integer(i) => format!("{}i", i),
                    FieldValue::Float(f) => format!("{}", f),
                    FieldValue::Boolean(b) => format!("{}", b),
                };
                format!("{}={}", k, value)
            })
            .collect();

        let timestamp_ns = self.timestamp.timestamp_nanos_opt().unwrap_or(0);

        if tags.is_empty() {
            format!("{} {} {}", self.measurement, fields.join(","), timestamp_ns)
        } else {
            format!(
                "{},{} {} {}",
                self.measurement,
                tags.join(","),
                fields.join(","),
                timestamp_ns
            )
        }
    }
}

/// Escape special characters in tag keys/values
fn escape_tag(s: &str) -> String {
    s.replace(' ', "\\ ")
        .replace(',', "\\,")
        .replace('=', "\\=")
}

/// Escape special characters in string field values
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// InfluxDB event store for time-series metrics
#[derive(Clone)]
pub struct InfluxEventStore {
    client: reqwest::Client,
    url: String,
    org: String,
    bucket: String,
    token: String,
}

impl InfluxEventStore {
    /// Create a new InfluxDB event store
    pub fn new(url: String, org: String, bucket: String, token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            org,
            bucket,
            token,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("INFLUXDB_URL").ok()?;
        let org = std::env::var("INFLUXDB_ORG").ok()?;
        let bucket = std::env::var("INFLUXDB_BUCKET").ok()?;
        let token = std::env::var("INFLUXDB_TOKEN").ok()?;

        Some(Self::new(url, org, bucket, token))
    }

    /// Write a single event to InfluxDB
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn write_event(&self, event: &Event) -> Result<()> {
        let metrics = EventMetrics::from_event(event);
        self.write_metrics(&metrics).await
    }

    /// Write multiple events to InfluxDB in batch
    #[instrument(skip(self, events), fields(count = events.len()))]
    pub async fn write_events(&self, events: &[Event]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let lines: Vec<String> = events
            .iter()
            .map(|e| EventMetrics::from_event(e).to_line_protocol())
            .collect();

        self.write_lines(&lines.join("\n")).await
    }

    /// Write raw metrics
    #[instrument(skip(self, metrics))]
    pub async fn write_metrics(&self, metrics: &EventMetrics) -> Result<()> {
        let line = metrics.to_line_protocol();
        self.write_lines(&line).await
    }

    /// Write line protocol data
    async fn write_lines(&self, data: &str) -> Result<()> {
        let url = format!(
            "{}/api/v2/write?org={}&bucket={}&precision=ns",
            self.url, self.org, self.bucket
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(data.to_string())
            .send()
            .await
            .map_err(|e| EventError::InfluxDb {
                message: format!("Failed to send request: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "InfluxDB write failed");
            return Err(EventError::InfluxDb {
                message: format!("InfluxDB write failed with status {}: {}", status, body),
            });
        }

        info!("Event metrics written to InfluxDB");
        Ok(())
    }

    /// Query events from InfluxDB using Flux
    #[instrument(skip(self))]
    pub async fn query(&self, flux_query: &str) -> Result<String> {
        let url = format!("{}/api/v2/query?org={}", self.url, self.org);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .header("Content-Type", "application/vnd.flux")
            .header("Accept", "application/csv")
            .body(flux_query.to_string())
            .send()
            .await
            .map_err(|e| EventError::InfluxDb {
                message: format!("Failed to send query: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventError::InfluxDb {
                message: format!("InfluxDB query failed with status {}: {}", status, body),
            });
        }

        response.text().await.map_err(|e| EventError::InfluxDb {
            message: format!("Failed to read response: {}", e),
        })
    }

    /// Get event count by category for a time range
    pub async fn count_by_category(&self, start: &str, stop: &str) -> Result<String> {
        let query = format!(
            r#"from(bucket: "{}")
                |> range(start: {}, stop: {})
                |> filter(fn: (r) => r._measurement == "events")
                |> group(columns: ["category"])
                |> count()
                |> yield(name: "count")"#,
            self.bucket, start, stop
        );

        self.query(&query).await
    }

    /// Get event count by severity for a time range
    pub async fn count_by_severity(&self, start: &str, stop: &str) -> Result<String> {
        let query = format!(
            r#"from(bucket: "{}")
                |> range(start: {}, stop: {})
                |> filter(fn: (r) => r._measurement == "events")
                |> group(columns: ["severity"])
                |> count()
                |> yield(name: "count")"#,
            self.bucket, start, stop
        );

        self.query(&query).await
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| EventError::InfluxDb {
                message: format!("Health check failed: {}", e),
            })?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_protocol_generation() {
        let mut tags = HashMap::new();
        tags.insert("category".to_string(), "user".to_string());

        let mut fields = HashMap::new();
        fields.insert("count".to_string(), FieldValue::Integer(1));

        let metrics = EventMetrics {
            measurement: "events".to_string(),
            tags,
            fields,
            timestamp: Utc::now(),
        };

        let line = metrics.to_line_protocol();
        assert!(line.starts_with("events,category=user count=1i"));
    }

    #[test]
    fn test_metrics_from_event() {
        use crate::models::Event;

        let event = Event::new(
            "user.login",
            EventCategory::User,
            EventSeverity::Info,
            "User logged in successfully",
        )
        .with_source("auth-service");

        let metrics = EventMetrics::from_event(&event);

        assert_eq!(metrics.measurement, "events");
        assert_eq!(metrics.tags.get("category"), Some(&"user".to_string()));
        assert_eq!(metrics.tags.get("severity"), Some(&"info".to_string()));
        assert_eq!(
            metrics.tags.get("source"),
            Some(&"auth-service".to_string())
        );
    }
}

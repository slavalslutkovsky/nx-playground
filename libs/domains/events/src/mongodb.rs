//! MongoDB implementation of EventRepository

use crate::error::{EventError, Result};
use crate::models::{Event, EventCategory, EventFilter, EventSeverity, EventStats};
use crate::repository::EventRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use futures_util::TryStreamExt;
use mongodb::bson::{Bson, Document, doc, to_bson};
use mongodb::{Collection, Database};
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

/// MongoDB-based event repository
#[derive(Clone)]
pub struct MongoEventRepository {
    collection: Collection<Event>,
}

impl MongoEventRepository {
    /// Create a new MongoDB event repository
    pub fn new(database: &Database) -> Self {
        Self {
            collection: database.collection("events"),
        }
    }

    /// Convert chrono DateTime to BSON DateTime
    fn to_bson_datetime(dt: chrono::DateTime<chrono::Utc>) -> Bson {
        Bson::DateTime(mongodb::bson::DateTime::from_millis(dt.timestamp_millis()))
    }

    /// Create indexes for efficient querying
    pub async fn create_indexes(&self) -> Result<()> {
        use mongodb::IndexModel;
        use mongodb::options::IndexOptions;

        let indexes = vec![
            // Index on timestamp for time-range queries
            IndexModel::builder().keys(doc! { "timestamp": -1 }).build(),
            // Compound index for common filters
            IndexModel::builder()
                .keys(doc! { "category": 1, "severity": 1, "timestamp": -1 })
                .build(),
            // Index on tags for tag-based queries
            IndexModel::builder().keys(doc! { "tags": 1 }).build(),
            // Index on source for source-based queries
            IndexModel::builder()
                .keys(doc! { "metadata.source": 1, "timestamp": -1 })
                .build(),
            // Index on correlation_id for tracing
            IndexModel::builder()
                .keys(doc! { "metadata.correlation_id": 1 })
                .build(),
            // TTL index for automatic cleanup (30 days retention)
            IndexModel::builder()
                .keys(doc! { "created_at": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(std::time::Duration::from_secs(30 * 24 * 60 * 60))
                        .build(),
                )
                .build(),
        ];

        self.collection.create_indexes(indexes).await?;
        Ok(())
    }

    /// Build filter document from EventFilter
    fn build_filter(&self, filter: &EventFilter) -> Document {
        let mut doc = Document::new();

        if let Some(category) = &filter.category {
            doc.insert("category", category.to_string());
        }

        if let Some(severity) = &filter.severity {
            doc.insert("severity", severity.to_string());
        }

        if let Some(min_severity) = &filter.min_severity {
            let severities = Self::get_severities_from(*min_severity);
            doc.insert("severity", doc! { "$in": severities });
        }

        if let Some(name) = &filter.name {
            doc.insert("name", name);
        }

        if let Some(source) = &filter.source {
            doc.insert("metadata.source", source);
        }

        if let Some(correlation_id) = &filter.correlation_id {
            doc.insert("metadata.correlation_id", correlation_id);
        }

        if let Some(tags) = &filter.tags {
            if !tags.is_empty() {
                doc.insert("tags", doc! { "$in": tags });
            }
        }

        // Time range filter
        let mut timestamp_filter = Document::new();
        if let Some(from) = filter.from {
            timestamp_filter.insert("$gte", Self::to_bson_datetime(from));
        }
        if let Some(to) = filter.to {
            timestamp_filter.insert("$lte", Self::to_bson_datetime(to));
        }
        if !timestamp_filter.is_empty() {
            doc.insert("timestamp", timestamp_filter);
        }

        // Full-text search in message
        if let Some(search) = &filter.search {
            let regex = format!("(?i){}", regex::escape(search));
            doc.insert("message", doc! { "$regex": regex });
        }

        doc
    }

    /// Get severity levels from a minimum severity (inclusive)
    fn get_severities_from(min: EventSeverity) -> Vec<String> {
        let all = [
            EventSeverity::Debug,
            EventSeverity::Info,
            EventSeverity::Warning,
            EventSeverity::Error,
            EventSeverity::Critical,
        ];

        let start = match min {
            EventSeverity::Debug => 0,
            EventSeverity::Info => 1,
            EventSeverity::Warning => 2,
            EventSeverity::Error => 3,
            EventSeverity::Critical => 4,
        };

        all[start..].iter().map(|s| s.to_string()).collect()
    }
}

#[async_trait]
impl EventRepository for MongoEventRepository {
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    async fn create(&self, event: Event) -> Result<Event> {
        self.collection.insert_one(&event).await?;
        Ok(event)
    }

    #[instrument(skip(self, events), fields(count = events.len()))]
    async fn create_batch(&self, events: Vec<Event>) -> Result<Vec<Event>> {
        if events.is_empty() {
            return Ok(vec![]);
        }

        self.collection.insert_many(&events).await?;
        Ok(events)
    }

    #[instrument(skip(self))]
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<Event>> {
        let filter = doc! { "_id": to_bson(id)? };
        let event = self.collection.find_one(filter).await?;
        Ok(event)
    }

    #[instrument(skip(self, filter))]
    async fn list(&self, filter: &EventFilter) -> Result<Vec<Event>> {
        use mongodb::options::FindOptions;

        let query = self.build_filter(filter);
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .skip(filter.offset)
            .limit(filter.limit as i64)
            .build();

        let cursor = self.collection.find(query).with_options(options).await?;
        let events: Vec<Event> = cursor.try_collect().await?;
        Ok(events)
    }

    #[instrument(skip(self, filter))]
    async fn count(&self, filter: &EventFilter) -> Result<u64> {
        let query = self.build_filter(filter);
        let count = self.collection.count_documents(query).await?;
        Ok(count)
    }

    #[instrument(skip(self))]
    async fn stats(&self) -> Result<EventStats> {
        let total = self.collection.count_documents(doc! {}).await?;

        // Count by category
        let mut by_category = HashMap::new();
        for category in [
            EventCategory::System,
            EventCategory::User,
            EventCategory::Api,
            EventCategory::Business,
            EventCategory::Security,
            EventCategory::Performance,
            EventCategory::Integration,
            EventCategory::Custom,
        ] {
            let count = self
                .collection
                .count_documents(doc! { "category": category.to_string() })
                .await?;
            if count > 0 {
                by_category.insert(category.to_string(), count);
            }
        }

        // Count by severity
        let mut by_severity = HashMap::new();
        for severity in [
            EventSeverity::Debug,
            EventSeverity::Info,
            EventSeverity::Warning,
            EventSeverity::Error,
            EventSeverity::Critical,
        ] {
            let count = self
                .collection
                .count_documents(doc! { "severity": severity.to_string() })
                .await?;
            if count > 0 {
                by_severity.insert(severity.to_string(), count);
            }
        }

        // Last hour
        let one_hour_ago = Utc::now() - Duration::hours(1);
        let last_hour = self
            .collection
            .count_documents(doc! { "timestamp": { "$gte": Self::to_bson_datetime(one_hour_ago) } })
            .await?;

        // Last 24 hours
        let one_day_ago = Utc::now() - Duration::hours(24);
        let last_24h = self
            .collection
            .count_documents(doc! { "timestamp": { "$gte": Self::to_bson_datetime(one_day_ago) } })
            .await?;

        Ok(EventStats {
            total,
            by_category,
            by_severity,
            last_hour,
            last_24h,
        })
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let filter = doc! { "_id": to_bson(id)? };
        let result = self.collection.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }

    #[instrument(skip(self))]
    async fn delete_before(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let filter = doc! { "timestamp": { "$lt": Self::to_bson_datetime(before) } };
        let result = self.collection.delete_many(filter).await?;
        Ok(result.deleted_count)
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Item status
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    Default,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ItemStatus {
    /// Item is active
    #[default]
    Active,
    /// Item is inactive/disabled
    Inactive,
    /// Item is archived
    Archived,
}

/// Item priority
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    Default,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ItemPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Item entity - represents an item stored in MongoDB
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Item {
    /// Unique identifier (stored as _id in MongoDB)
    #[serde(rename = "_id", alias = "id")]
    pub id: Uuid,
    /// Item name
    pub name: String,
    /// Item description
    pub description: String,
    /// Current status
    pub status: ItemStatus,
    /// Priority level
    pub priority: ItemPriority,
    /// Optional category
    pub category: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Quantity (for inventory-like items)
    pub quantity: i32,
    /// Price (optional)
    pub price: Option<f64>,
    /// Additional metadata as JSON
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new item
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateItem {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: ItemStatus,
    #[serde(default)]
    pub priority: ItemPriority,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    #[validate(range(min = 0))]
    pub quantity: i32,
    #[validate(range(min = 0.0))]
    pub price: Option<f64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// DTO for updating an existing item
#[derive(Debug, Clone, Default, Deserialize, Validate, ToSchema)]
pub struct UpdateItem {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ItemStatus>,
    pub priority: Option<ItemPriority>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    #[validate(range(min = 0))]
    pub quantity: Option<i32>,
    #[validate(range(min = 0.0))]
    pub price: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

/// Query filters for listing items
#[derive(Debug, Clone, Default, Deserialize, ToSchema, IntoParams)]
pub struct ItemFilter {
    /// Filter by status
    pub status: Option<ItemStatus>,
    /// Filter by priority
    pub priority: Option<ItemPriority>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by tag (items containing this tag)
    pub tag: Option<String>,
    /// Search in name and description
    pub search: Option<String>,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip
    #[serde(default)]
    pub offset: u64,
}

fn default_limit() -> i64 {
    50
}

impl Item {
    /// Create a new item from CreateItem DTO
    pub fn new(input: CreateItem) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: input.name,
            description: input.description,
            status: input.status,
            priority: input.priority,
            category: input.category,
            tags: input.tags,
            quantity: input.quantity,
            price: input.price,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Apply updates from UpdateItem DTO
    pub fn apply_update(&mut self, update: UpdateItem) {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(description) = update.description {
            self.description = description;
        }
        if let Some(status) = update.status {
            self.status = status;
        }
        if let Some(priority) = update.priority {
            self.priority = priority;
        }
        if let Some(category) = update.category {
            self.category = Some(category);
        }
        if let Some(tags) = update.tags {
            self.tags = tags;
        }
        if let Some(quantity) = update.quantity {
            self.quantity = quantity;
        }
        if let Some(price) = update.price {
            self.price = Some(price);
        }
        if let Some(metadata) = update.metadata {
            self.metadata = metadata;
        }
        self.updated_at = Utc::now();
    }
}

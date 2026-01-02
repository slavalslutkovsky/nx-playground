use async_trait::async_trait;
use uuid::Uuid;

use crate::error::ItemResult;
use crate::models::{CreateItem, Item, ItemFilter, UpdateItem};

/// Repository trait for Item persistence
///
/// This trait defines the data access interface for items.
/// Implementations can use different storage backends (MongoDB, etc.)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ItemRepository: Send + Sync {
    /// Create a new item
    async fn create(&self, input: CreateItem) -> ItemResult<Item>;

    /// Get an item by ID
    async fn get_by_id(&self, id: Uuid) -> ItemResult<Option<Item>>;

    /// List items with optional filters
    async fn list(&self, filter: ItemFilter) -> ItemResult<Vec<Item>>;

    /// Update an existing item
    async fn update(&self, id: Uuid, input: UpdateItem) -> ItemResult<Item>;

    /// Delete an item by ID
    async fn delete(&self, id: Uuid) -> ItemResult<bool>;

    /// Count items matching a filter
    async fn count(&self, filter: ItemFilter) -> ItemResult<u64>;

    /// Check if an item name exists
    async fn exists_by_name(&self, name: &str) -> ItemResult<bool>;
}

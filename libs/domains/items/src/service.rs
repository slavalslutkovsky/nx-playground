//! Item Service - Business logic layer

use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ItemError, ItemResult};
use crate::models::{CreateItem, Item, ItemFilter, ItemStatus, UpdateItem};
use crate::repository::ItemRepository;

/// Item service providing business logic operations
///
/// The service layer handles validation, business rules, and orchestrates
/// repository operations.
pub struct ItemService<R: ItemRepository> {
    repository: Arc<R>,
}

impl<R: ItemRepository> ItemService<R> {
    /// Create a new ItemService with the given repository
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Create a new item
    #[instrument(skip(self, input), fields(item_name = %input.name))]
    pub async fn create_item(&self, input: CreateItem) -> ItemResult<Item> {
        // Validate input
        input
            .validate()
            .map_err(|e| ItemError::Validation(e.to_string()))?;

        // Check for duplicate name
        if self.repository.exists_by_name(&input.name).await? {
            return Err(ItemError::DuplicateName(input.name));
        }

        self.repository.create(input).await
    }

    /// Get an item by ID
    #[instrument(skip(self))]
    pub async fn get_item(&self, id: Uuid) -> ItemResult<Item> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or(ItemError::NotFound(id))
    }

    /// List items with optional filters
    #[instrument(skip(self))]
    pub async fn list_items(&self, filter: ItemFilter) -> ItemResult<Vec<Item>> {
        self.repository.list(filter).await
    }

    /// Update an existing item
    #[instrument(skip(self, input))]
    pub async fn update_item(&self, id: Uuid, input: UpdateItem) -> ItemResult<Item> {
        // Validate input
        input
            .validate()
            .map_err(|e| ItemError::Validation(e.to_string()))?;

        // Check if item exists
        let existing = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(ItemError::NotFound(id))?;

        // Check for duplicate name if name is being changed
        if let Some(ref new_name) = input.name {
            if new_name != &existing.name && self.repository.exists_by_name(new_name).await? {
                return Err(ItemError::DuplicateName(new_name.clone()));
            }
        }

        self.repository.update(id, input).await
    }

    /// Delete an item
    #[instrument(skip(self))]
    pub async fn delete_item(&self, id: Uuid) -> ItemResult<()> {
        self.repository.delete(id).await?;
        Ok(())
    }

    /// Count items matching a filter
    #[instrument(skip(self))]
    pub async fn count_items(&self, filter: ItemFilter) -> ItemResult<u64> {
        self.repository.count(filter).await
    }

    /// Activate an item (set status to Active)
    #[instrument(skip(self))]
    pub async fn activate_item(&self, id: Uuid) -> ItemResult<Item> {
        let update = UpdateItem {
            status: Some(ItemStatus::Active),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }

    /// Deactivate an item (set status to Inactive)
    #[instrument(skip(self))]
    pub async fn deactivate_item(&self, id: Uuid) -> ItemResult<Item> {
        let update = UpdateItem {
            status: Some(ItemStatus::Inactive),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }

    /// Archive an item
    #[instrument(skip(self))]
    pub async fn archive_item(&self, id: Uuid) -> ItemResult<Item> {
        let update = UpdateItem {
            status: Some(ItemStatus::Archived),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }
}

impl<R: ItemRepository> Clone for ItemService<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
        }
    }
}

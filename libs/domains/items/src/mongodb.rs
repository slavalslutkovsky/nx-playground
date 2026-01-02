//! MongoDB implementation of ItemRepository

use async_trait::async_trait;
use mongodb::{
    Collection, Database,
    bson::{Bson, doc, to_bson},
};
use tracing::instrument;
use uuid::Uuid;

use crate::error::{ItemError, ItemResult};
use crate::models::{CreateItem, Item, ItemFilter, UpdateItem};
use crate::repository::ItemRepository;

/// MongoDB implementation of the ItemRepository
pub struct MongoItemRepository {
    collection: Collection<Item>,
}

impl MongoItemRepository {
    /// Create a new MongoItemRepository
    ///
    /// # Arguments
    /// * `db` - MongoDB database instance
    ///
    /// # Example
    /// ```ignore
    /// let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    /// let db = client.database("mydb");
    /// let repo = MongoItemRepository::new(db);
    /// ```
    pub fn new(db: Database) -> Self {
        let collection = db.collection::<Item>("items");
        Self { collection }
    }

    /// Create a new MongoItemRepository with a custom collection name
    pub fn with_collection(db: Database, collection_name: &str) -> Self {
        let collection = db.collection::<Item>(collection_name);
        Self { collection }
    }

    /// Get the underlying collection for advanced operations
    pub fn collection(&self) -> &Collection<Item> {
        &self.collection
    }

    /// Build a MongoDB filter document from ItemFilter
    fn build_filter(filter: &ItemFilter) -> mongodb::bson::Document {
        let mut doc = doc! {};

        if let Some(ref status) = filter.status {
            doc.insert("status", status.to_string());
        }

        if let Some(ref priority) = filter.priority {
            doc.insert("priority", priority.to_string());
        }

        if let Some(ref category) = filter.category {
            doc.insert("category", category);
        }

        if let Some(ref tag) = filter.tag {
            doc.insert("tags", doc! { "$in": [tag] });
        }

        if let Some(ref search) = filter.search {
            doc.insert(
                "$or",
                vec![
                    doc! { "name": { "$regex": search, "$options": "i" } },
                    doc! { "description": { "$regex": search, "$options": "i" } },
                ],
            );
        }

        doc
    }
}

#[async_trait]
impl ItemRepository for MongoItemRepository {
    #[instrument(skip(self, input), fields(item_name = %input.name))]
    async fn create(&self, input: CreateItem) -> ItemResult<Item> {
        let item = Item::new(input);

        self.collection.insert_one(&item).await?;

        tracing::info!(item_id = %item.id, "Item created successfully");
        Ok(item)
    }

    #[instrument(skip(self))]
    async fn get_by_id(&self, id: Uuid) -> ItemResult<Option<Item>> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let item = self.collection.find_one(filter).await?;
        Ok(item)
    }

    #[instrument(skip(self))]
    async fn list(&self, filter: ItemFilter) -> ItemResult<Vec<Item>> {
        use futures_util::TryStreamExt;

        let mongo_filter = Self::build_filter(&filter);

        let options = mongodb::options::FindOptions::builder()
            .limit(filter.limit)
            .skip(filter.offset)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .collection
            .find(mongo_filter)
            .with_options(options)
            .await?;
        let items: Vec<Item> = cursor.try_collect().await?;

        Ok(items)
    }

    #[instrument(skip(self, input))]
    async fn update(&self, id: Uuid, input: UpdateItem) -> ItemResult<Item> {
        // First, get the existing item
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let existing = self
            .collection
            .find_one(filter.clone())
            .await?
            .ok_or(ItemError::NotFound(id))?;

        // Apply updates
        let mut updated = existing;
        updated.apply_update(input);

        // Replace the document
        self.collection.replace_one(filter, &updated).await?;

        tracing::info!(item_id = %id, "Item updated successfully");
        Ok(updated)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> ItemResult<bool> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let result = self.collection.delete_one(filter).await?;

        if result.deleted_count == 0 {
            return Err(ItemError::NotFound(id));
        }

        tracing::info!(item_id = %id, "Item deleted successfully");
        Ok(true)
    }

    #[instrument(skip(self))]
    async fn count(&self, filter: ItemFilter) -> ItemResult<u64> {
        let mongo_filter = Self::build_filter(&filter);
        let count = self.collection.count_documents(mongo_filter).await?;
        Ok(count)
    }

    #[instrument(skip(self))]
    async fn exists_by_name(&self, name: &str) -> ItemResult<bool> {
        let filter = doc! { "name": name };
        let count = self.collection.count_documents(filter).await?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a MongoDB instance
    // These are placeholder tests that verify the code compiles

    #[test]
    fn test_build_filter_empty() {
        let filter = ItemFilter::default();
        let doc = MongoItemRepository::build_filter(&filter);
        assert!(doc.is_empty());
    }

    #[test]
    fn test_build_filter_with_status() {
        use crate::models::ItemStatus;
        let filter = ItemFilter {
            status: Some(ItemStatus::Active),
            ..Default::default()
        };
        let doc = MongoItemRepository::build_filter(&filter);
        assert!(doc.contains_key("status"));
    }

    #[test]
    fn test_build_filter_with_search() {
        let filter = ItemFilter {
            search: Some("test".to_string()),
            ..Default::default()
        };
        let doc = MongoItemRepository::build_filter(&filter);
        assert!(doc.contains_key("$or"));
    }
}

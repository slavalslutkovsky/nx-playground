//! MongoDB implementation of ProductRepository

use async_trait::async_trait;
use mongodb::{
    bson::{doc, to_bson, Bson},
    options::IndexOptions,
    Collection, Database, IndexModel,
};
use tracing::instrument;
use uuid::Uuid;

use crate::error::{ProductError, ProductResult};
use crate::models::{CreateProduct, Product, ProductFilter, ProductStatus, UpdateProduct};
use crate::repository::ProductRepository;

/// MongoDB implementation of the ProductRepository
pub struct MongoProductRepository {
    collection: Collection<Product>,
}

impl MongoProductRepository {
    /// Create a new MongoProductRepository
    pub fn new(db: &Database) -> Self {
        let collection = db.collection::<Product>("products");
        Self { collection }
    }

    /// Create a new MongoProductRepository with a custom collection name
    pub fn with_collection(db: &Database, collection_name: &str) -> Self {
        let collection = db.collection::<Product>(collection_name);
        Self { collection }
    }

    /// Initialize indexes for optimal query performance
    pub async fn init_indexes(&self) -> ProductResult<()> {
        let indexes = vec![
            // Unique SKU index
            IndexModel::builder()
                .keys(doc! { "sku": 1 })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .sparse(true)
                        .name("idx_sku_unique".to_string())
                        .build(),
                )
                .build(),
            // Unique barcode index
            IndexModel::builder()
                .keys(doc! { "barcode": 1 })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .sparse(true)
                        .name("idx_barcode_unique".to_string())
                        .build(),
                )
                .build(),
            // Category + status for listing
            IndexModel::builder()
                .keys(doc! { "category": 1, "status": 1, "created_at": -1 })
                .options(
                    IndexOptions::builder()
                        .name("idx_category_status".to_string())
                        .build(),
                )
                .build(),
            // Price range queries
            IndexModel::builder()
                .keys(doc! { "price": 1 })
                .options(
                    IndexOptions::builder()
                        .name("idx_price".to_string())
                        .build(),
                )
                .build(),
            // Stock level queries
            IndexModel::builder()
                .keys(doc! { "stock": 1 })
                .options(
                    IndexOptions::builder()
                        .name("idx_stock".to_string())
                        .build(),
                )
                .build(),
            // Text search on name and description
            IndexModel::builder()
                .keys(doc! { "name": "text", "description": "text", "tags": "text" })
                .options(
                    IndexOptions::builder()
                        .name("idx_text_search".to_string())
                        .build(),
                )
                .build(),
            // Tags index
            IndexModel::builder()
                .keys(doc! { "tags": 1 })
                .options(IndexOptions::builder().name("idx_tags".to_string()).build())
                .build(),
            // Brand index
            IndexModel::builder()
                .keys(doc! { "brand": 1 })
                .options(
                    IndexOptions::builder()
                        .name("idx_brand".to_string())
                        .build(),
                )
                .build(),
        ];

        self.collection.create_indexes(indexes).await?;
        tracing::info!("Product indexes created successfully");
        Ok(())
    }

    /// Get the underlying collection for advanced operations
    pub fn collection(&self) -> &Collection<Product> {
        &self.collection
    }

    /// Build a MongoDB filter document from ProductFilter
    fn build_filter(filter: &ProductFilter) -> mongodb::bson::Document {
        let mut doc = doc! {};

        if let Some(ref status) = filter.status {
            doc.insert("status", status.to_string());
        }

        if let Some(ref category) = filter.category {
            doc.insert("category", category.to_string());
        }

        if let Some(ref brand) = filter.brand {
            doc.insert("brand", brand);
        }

        // Price range
        if filter.min_price.is_some() || filter.max_price.is_some() {
            let mut price_filter = doc! {};
            if let Some(min) = filter.min_price {
                price_filter.insert("$gte", min);
            }
            if let Some(max) = filter.max_price {
                price_filter.insert("$lte", max);
            }
            doc.insert("price", price_filter);
        }

        // In stock filter
        if let Some(in_stock) = filter.in_stock {
            if in_stock {
                doc.insert(
                    "$expr",
                    doc! {
                        "$gt": ["$stock", "$reserved_stock"]
                    },
                );
            }
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
impl ProductRepository for MongoProductRepository {
    #[instrument(skip(self, input), fields(product_name = %input.name))]
    async fn create(&self, input: CreateProduct) -> ProductResult<Product> {
        let product = Product::new(input);

        self.collection.insert_one(&product).await?;

        tracing::info!(product_id = %product.id, "Product created successfully");
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn get_by_id(&self, id: Uuid) -> ProductResult<Option<Product>> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let product = self.collection.find_one(filter).await?;
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn get_by_sku(&self, sku: &str) -> ProductResult<Option<Product>> {
        let filter = doc! { "sku": sku };
        let product = self.collection.find_one(filter).await?;
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn get_by_barcode(&self, barcode: &str) -> ProductResult<Option<Product>> {
        let filter = doc! { "barcode": barcode };
        let product = self.collection.find_one(filter).await?;
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn list(&self, filter: ProductFilter) -> ProductResult<Vec<Product>> {
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
        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    #[instrument(skip(self))]
    async fn search(&self, query: &str, limit: i64, offset: u64) -> ProductResult<Vec<Product>> {
        use futures_util::TryStreamExt;

        let filter = doc! {
            "$text": { "$search": query }
        };

        let options = mongodb::options::FindOptions::builder()
            .limit(limit)
            .skip(offset)
            .sort(doc! { "score": { "$meta": "textScore" } })
            .build();

        let cursor = self.collection.find(filter).with_options(options).await?;
        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    #[instrument(skip(self, input))]
    async fn update(&self, id: Uuid, input: UpdateProduct) -> ProductResult<Product> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let existing = self
            .collection
            .find_one(filter.clone())
            .await?
            .ok_or(ProductError::NotFound(id))?;

        let mut updated = existing;
        updated.apply_update(input);

        self.collection.replace_one(filter, &updated).await?;

        tracing::info!(product_id = %id, "Product updated successfully");
        Ok(updated)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> ProductResult<bool> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };
        let result = self.collection.delete_one(filter).await?;

        if result.deleted_count == 0 {
            return Err(ProductError::NotFound(id));
        }

        tracing::info!(product_id = %id, "Product deleted successfully");
        Ok(true)
    }

    #[instrument(skip(self))]
    async fn count(&self, filter: ProductFilter) -> ProductResult<u64> {
        let mongo_filter = Self::build_filter(&filter);
        let count = self.collection.count_documents(mongo_filter).await?;
        Ok(count)
    }

    #[instrument(skip(self))]
    async fn exists_by_sku(&self, sku: &str) -> ProductResult<bool> {
        let filter = doc! { "sku": sku };
        let count = self.collection.count_documents(filter).await?;
        Ok(count > 0)
    }

    #[instrument(skip(self))]
    async fn exists_by_name(&self, name: &str) -> ProductResult<bool> {
        let filter = doc! { "name": name };
        let count = self.collection.count_documents(filter).await?;
        Ok(count > 0)
    }

    #[instrument(skip(self))]
    async fn update_stock(&self, id: Uuid, quantity_change: i32) -> ProductResult<Product> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };

        let update = doc! {
            "$inc": { "stock": quantity_change },
            "$set": { "updated_at": chrono::Utc::now().to_rfc3339() }
        };

        self.collection.update_one(filter.clone(), update).await?;

        // Fetch and return updated product
        let product = self
            .collection
            .find_one(filter)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        // Update status based on stock
        if product.stock == 0 && product.status == ProductStatus::Active {
            let status_update = doc! {
                "$set": { "status": "out_of_stock" }
            };
            self.collection
                .update_one(
                    doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) },
                    status_update,
                )
                .await?;
        }

        tracing::info!(product_id = %id, quantity_change, "Stock updated");
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn reserve_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };

        let update = doc! {
            "$inc": { "reserved_stock": quantity },
            "$set": { "updated_at": chrono::Utc::now().to_rfc3339() }
        };

        self.collection.update_one(filter.clone(), update).await?;

        let product = self
            .collection
            .find_one(filter)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        tracing::info!(product_id = %id, quantity, "Stock reserved");
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn release_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };

        let update = doc! {
            "$inc": { "reserved_stock": -quantity },
            "$set": { "updated_at": chrono::Utc::now().to_rfc3339() }
        };

        self.collection.update_one(filter.clone(), update).await?;

        let product = self
            .collection
            .find_one(filter)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        tracing::info!(product_id = %id, quantity, "Stock released");
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn commit_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product> {
        let filter = doc! { "_id": to_bson(&id).unwrap_or(Bson::Null) };

        let update = doc! {
            "$inc": {
                "stock": -quantity,
                "reserved_stock": -quantity
            },
            "$set": { "updated_at": chrono::Utc::now().to_rfc3339() }
        };

        self.collection.update_one(filter.clone(), update).await?;

        let product = self
            .collection
            .find_one(filter)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        tracing::info!(product_id = %id, quantity, "Stock committed");
        Ok(product)
    }

    #[instrument(skip(self))]
    async fn get_by_category(
        &self,
        category: &str,
        limit: i64,
        offset: u64,
    ) -> ProductResult<Vec<Product>> {
        use futures_util::TryStreamExt;

        let filter = doc! {
            "category": category,
            "status": { "$ne": "discontinued" }
        };

        let options = mongodb::options::FindOptions::builder()
            .limit(limit)
            .skip(offset)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self.collection.find(filter).with_options(options).await?;
        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    #[instrument(skip(self))]
    async fn get_low_stock(&self, threshold: i32, limit: i64) -> ProductResult<Vec<Product>> {
        use futures_util::TryStreamExt;

        let filter = doc! {
            "stock": { "$lte": threshold },
            "status": { "$in": ["active", "out_of_stock"] }
        };

        let options = mongodb::options::FindOptions::builder()
            .limit(limit)
            .sort(doc! { "stock": 1 })
            .build();

        let cursor = self.collection.find(filter).with_options(options).await?;
        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProductCategory;

    #[test]
    fn test_build_filter_empty() {
        let filter = ProductFilter::default();
        let doc = MongoProductRepository::build_filter(&filter);
        assert!(doc.is_empty());
    }

    #[test]
    fn test_build_filter_with_status() {
        let filter = ProductFilter {
            status: Some(ProductStatus::Active),
            ..Default::default()
        };
        let doc = MongoProductRepository::build_filter(&filter);
        assert!(doc.contains_key("status"));
    }

    #[test]
    fn test_build_filter_with_category() {
        let filter = ProductFilter {
            category: Some(ProductCategory::Electronics),
            ..Default::default()
        };
        let doc = MongoProductRepository::build_filter(&filter);
        assert!(doc.contains_key("category"));
    }

    #[test]
    fn test_build_filter_with_price_range() {
        let filter = ProductFilter {
            min_price: Some(1000),
            max_price: Some(5000),
            ..Default::default()
        };
        let doc = MongoProductRepository::build_filter(&filter);
        assert!(doc.contains_key("price"));
    }

    #[test]
    fn test_build_filter_with_search() {
        let filter = ProductFilter {
            search: Some("laptop".to_string()),
            ..Default::default()
        };
        let doc = MongoProductRepository::build_filter(&filter);
        assert!(doc.contains_key("$or"));
    }
}

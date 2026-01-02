use async_trait::async_trait;
use uuid::Uuid;

use crate::error::ProductResult;
use crate::models::{CreateProduct, Product, ProductFilter, UpdateProduct};

/// Repository trait for Product persistence
///
/// This trait defines the data access interface for products.
/// Implementations can use different storage backends (MongoDB, PostgreSQL, etc.)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Create a new product
    async fn create(&self, input: CreateProduct) -> ProductResult<Product>;

    /// Get a product by ID
    async fn get_by_id(&self, id: Uuid) -> ProductResult<Option<Product>>;

    /// Get a product by SKU
    async fn get_by_sku(&self, sku: &str) -> ProductResult<Option<Product>>;

    /// Get a product by barcode
    async fn get_by_barcode(&self, barcode: &str) -> ProductResult<Option<Product>>;

    /// List products with optional filters
    async fn list(&self, filter: ProductFilter) -> ProductResult<Vec<Product>>;

    /// Search products by text query
    async fn search(&self, query: &str, limit: i64, offset: u64) -> ProductResult<Vec<Product>>;

    /// Update an existing product
    async fn update(&self, id: Uuid, input: UpdateProduct) -> ProductResult<Product>;

    /// Delete a product by ID
    async fn delete(&self, id: Uuid) -> ProductResult<bool>;

    /// Count products matching a filter
    async fn count(&self, filter: ProductFilter) -> ProductResult<u64>;

    /// Check if a product SKU exists
    async fn exists_by_sku(&self, sku: &str) -> ProductResult<bool>;

    /// Check if a product name exists
    async fn exists_by_name(&self, name: &str) -> ProductResult<bool>;

    /// Update product stock
    async fn update_stock(&self, id: Uuid, quantity_change: i32) -> ProductResult<Product>;

    /// Reserve stock for an order
    async fn reserve_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product>;

    /// Release reserved stock
    async fn release_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product>;

    /// Commit reserved stock (after order completion)
    async fn commit_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product>;

    /// Get products by category
    async fn get_by_category(
        &self,
        category: &str,
        limit: i64,
        offset: u64,
    ) -> ProductResult<Vec<Product>>;

    /// Get low stock products (stock below threshold)
    async fn get_low_stock(&self, threshold: i32, limit: i64) -> ProductResult<Vec<Product>>;
}

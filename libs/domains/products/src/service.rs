//! Product Service - Business logic layer

use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ProductError, ProductResult};
use crate::models::{
    CreateProduct, Product, ProductCategory, ProductFilter, ProductStatus, ReservationResult,
    StockAdjustment, StockReservation, UpdateProduct,
};
use crate::repository::ProductRepository;

/// Product service providing business logic operations
///
/// The service layer handles validation, business rules, and orchestrates
/// repository operations.
pub struct ProductService<R: ProductRepository> {
    repository: Arc<R>,
}

impl<R: ProductRepository> ProductService<R> {
    /// Create a new ProductService with the given repository
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Create a new product
    #[instrument(skip(self, input), fields(product_name = %input.name))]
    pub async fn create_product(&self, input: CreateProduct) -> ProductResult<Product> {
        // Validate input
        input
            .validate()
            .map_err(|e| ProductError::Validation(e.to_string()))?;

        // Check for duplicate SKU if provided
        if let Some(ref sku) = input.sku {
            if self.repository.exists_by_sku(sku).await? {
                return Err(ProductError::DuplicateSku(sku.clone()));
            }
        }

        // Check for duplicate name
        if self.repository.exists_by_name(&input.name).await? {
            return Err(ProductError::DuplicateName(input.name.clone()));
        }

        self.repository.create(input).await
    }

    /// Get a product by ID
    #[instrument(skip(self))]
    pub async fn get_product(&self, id: Uuid) -> ProductResult<Product> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or(ProductError::NotFound(id))
    }

    /// Get a product by SKU
    #[instrument(skip(self))]
    pub async fn get_by_sku(&self, sku: &str) -> ProductResult<Product> {
        self.repository.get_by_sku(sku).await?.ok_or_else(|| {
            ProductError::Validation(format!("Product with SKU '{}' not found", sku))
        })
    }

    /// Get a product by barcode
    #[instrument(skip(self))]
    pub async fn get_by_barcode(&self, barcode: &str) -> ProductResult<Product> {
        self.repository
            .get_by_barcode(barcode)
            .await?
            .ok_or_else(|| {
                ProductError::Validation(format!("Product with barcode '{}' not found", barcode))
            })
    }

    /// List products with optional filters
    #[instrument(skip(self))]
    pub async fn list_products(&self, filter: ProductFilter) -> ProductResult<Vec<Product>> {
        self.repository.list(filter).await
    }

    /// Search products
    #[instrument(skip(self))]
    pub async fn search_products(
        &self,
        query: &str,
        limit: i64,
        offset: u64,
    ) -> ProductResult<Vec<Product>> {
        self.repository.search(query, limit, offset).await
    }

    /// Update an existing product
    #[instrument(skip(self, input))]
    pub async fn update_product(&self, id: Uuid, input: UpdateProduct) -> ProductResult<Product> {
        // Validate input
        input
            .validate()
            .map_err(|e| ProductError::Validation(e.to_string()))?;

        // Check if product exists
        let existing = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        // Check for duplicate SKU if being changed
        if let Some(ref new_sku) = input.sku {
            if existing.sku.as_ref() != Some(new_sku)
                && self.repository.exists_by_sku(new_sku).await?
            {
                return Err(ProductError::DuplicateSku(new_sku.clone()));
            }
        }

        // Check for duplicate name if being changed
        if let Some(ref new_name) = input.name {
            if new_name != &existing.name && self.repository.exists_by_name(new_name).await? {
                return Err(ProductError::DuplicateName(new_name.clone()));
            }
        }

        self.repository.update(id, input).await
    }

    /// Delete a product
    #[instrument(skip(self))]
    pub async fn delete_product(&self, id: Uuid) -> ProductResult<()> {
        // Check if product exists
        if self.repository.get_by_id(id).await?.is_none() {
            return Err(ProductError::NotFound(id));
        }

        self.repository.delete(id).await?;
        Ok(())
    }

    /// Count products matching a filter
    #[instrument(skip(self))]
    pub async fn count_products(&self, filter: ProductFilter) -> ProductResult<u64> {
        self.repository.count(filter).await
    }

    /// Adjust product stock
    #[instrument(skip(self, adjustment))]
    pub async fn adjust_stock(
        &self,
        id: Uuid,
        adjustment: StockAdjustment,
    ) -> ProductResult<Product> {
        adjustment
            .validate()
            .map_err(|e| ProductError::Validation(e.to_string()))?;

        // Check if product exists
        let product = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        // Validate stock won't go negative
        let new_stock = product.stock + adjustment.quantity;
        if new_stock < 0 {
            return Err(ProductError::InsufficientStock {
                available: product.stock,
                requested: -adjustment.quantity,
            });
        }

        self.repository.update_stock(id, adjustment.quantity).await
    }

    /// Reserve stock for an order
    #[instrument(skip(self, reservation))]
    pub async fn reserve_stock(
        &self,
        id: Uuid,
        reservation: StockReservation,
    ) -> ProductResult<ReservationResult> {
        reservation
            .validate()
            .map_err(|e| ProductError::Validation(e.to_string()))?;

        let product = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        let available = product.available_stock();
        if available < reservation.quantity {
            return Ok(ReservationResult {
                reservation_id: String::new(),
                success: false,
                message: format!(
                    "Insufficient stock: {} available, {} requested",
                    available, reservation.quantity
                ),
            });
        }

        self.repository
            .reserve_stock(id, reservation.quantity)
            .await?;

        let reservation_id = format!("{}-{}", id, reservation.order_id);
        Ok(ReservationResult {
            reservation_id,
            success: true,
            message: "Stock reserved successfully".to_string(),
        })
    }

    /// Release reserved stock
    #[instrument(skip(self))]
    pub async fn release_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product> {
        // Check if product exists
        if self.repository.get_by_id(id).await?.is_none() {
            return Err(ProductError::NotFound(id));
        }

        self.repository.release_stock(id, quantity).await
    }

    /// Commit reserved stock (after order completion)
    #[instrument(skip(self))]
    pub async fn commit_stock(&self, id: Uuid, quantity: i32) -> ProductResult<Product> {
        let product = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(ProductError::NotFound(id))?;

        if product.reserved_stock < quantity {
            return Err(ProductError::InsufficientStock {
                available: product.reserved_stock,
                requested: quantity,
            });
        }

        self.repository.commit_stock(id, quantity).await
    }

    /// Get products by category
    #[instrument(skip(self))]
    pub async fn get_by_category(
        &self,
        category: ProductCategory,
        limit: i64,
        offset: u64,
    ) -> ProductResult<Vec<Product>> {
        self.repository
            .get_by_category(&category.to_string(), limit, offset)
            .await
    }

    /// Get low stock products
    #[instrument(skip(self))]
    pub async fn get_low_stock(&self, threshold: i32, limit: i64) -> ProductResult<Vec<Product>> {
        self.repository.get_low_stock(threshold, limit).await
    }

    /// Activate a product
    #[instrument(skip(self))]
    pub async fn activate_product(&self, id: Uuid) -> ProductResult<Product> {
        let update = UpdateProduct {
            status: Some(ProductStatus::Active),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }

    /// Deactivate a product
    #[instrument(skip(self))]
    pub async fn deactivate_product(&self, id: Uuid) -> ProductResult<Product> {
        let update = UpdateProduct {
            status: Some(ProductStatus::Inactive),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }

    /// Discontinue a product
    #[instrument(skip(self))]
    pub async fn discontinue_product(&self, id: Uuid) -> ProductResult<Product> {
        let update = UpdateProduct {
            status: Some(ProductStatus::Discontinued),
            ..Default::default()
        };
        self.repository.update(id, update).await
    }
}

impl<R: ProductRepository> Clone for ProductService<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
        }
    }
}

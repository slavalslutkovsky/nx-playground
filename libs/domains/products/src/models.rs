use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Product status
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
pub enum ProductStatus {
    /// Product is active and available for sale
    #[default]
    Active,
    /// Product is inactive/disabled
    Inactive,
    /// Product is out of stock
    OutOfStock,
    /// Product is discontinued
    Discontinued,
    /// Product is in draft state
    Draft,
}

/// Product category
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
pub enum ProductCategory {
    #[default]
    General,
    Electronics,
    Clothing,
    Food,
    Books,
    HomeGarden,
    Sports,
    Toys,
    Health,
    Automotive,
    Other,
}

/// Product image
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductImage {
    /// Image URL
    pub url: String,
    /// Alternative text for accessibility
    #[serde(default)]
    pub alt: Option<String>,
    /// Whether this is the primary/hero image
    #[serde(default)]
    pub is_primary: bool,
    /// Sort order for display
    #[serde(default)]
    pub sort_order: i32,
}

/// Product entity - represents a product stored in MongoDB
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    /// Unique identifier (stored as _id in MongoDB)
    #[serde(rename = "_id", alias = "id")]
    pub id: Uuid,
    /// Product name
    pub name: String,
    /// Product description
    pub description: String,
    /// Price in cents (for precision)
    pub price: i64,
    /// Display price (computed from price)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_price: Option<f64>,
    /// Current stock quantity
    pub stock: i32,
    /// Reserved stock (for pending orders)
    #[serde(default)]
    pub reserved_stock: i32,
    /// Product category
    pub category: ProductCategory,
    /// Current status
    pub status: ProductStatus,
    /// Product images
    #[serde(default)]
    pub images: Vec<ProductImage>,
    /// Stock Keeping Unit (unique product identifier)
    pub sku: Option<String>,
    /// Barcode (UPC, EAN, etc.)
    pub barcode: Option<String>,
    /// Brand name
    pub brand: Option<String>,
    /// Product weight in grams
    pub weight: Option<i32>,
    /// Tags for search and organization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Additional metadata as JSON
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new product
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateProduct {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Price in cents
    #[validate(range(min = 0))]
    pub price: i64,
    #[validate(range(min = 0))]
    #[serde(default)]
    pub stock: i32,
    #[serde(default)]
    pub category: ProductCategory,
    #[serde(default)]
    pub status: ProductStatus,
    #[serde(default)]
    pub images: Vec<ProductImage>,
    #[validate(length(max = 50))]
    pub sku: Option<String>,
    #[validate(length(max = 50))]
    pub barcode: Option<String>,
    pub brand: Option<String>,
    #[validate(range(min = 0))]
    pub weight: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// DTO for updating an existing product
#[derive(Debug, Clone, Default, Deserialize, Validate, ToSchema)]
pub struct UpdateProduct {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub price: Option<i64>,
    #[validate(range(min = 0))]
    pub stock: Option<i32>,
    pub category: Option<ProductCategory>,
    pub status: Option<ProductStatus>,
    pub images: Option<Vec<ProductImage>>,
    #[validate(length(max = 50))]
    pub sku: Option<String>,
    #[validate(length(max = 50))]
    pub barcode: Option<String>,
    pub brand: Option<String>,
    #[validate(range(min = 0))]
    pub weight: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Query filters for listing products
#[derive(Debug, Clone, Default, Deserialize, ToSchema, IntoParams)]
pub struct ProductFilter {
    /// Filter by status
    pub status: Option<ProductStatus>,
    /// Filter by category
    pub category: Option<ProductCategory>,
    /// Filter by brand
    pub brand: Option<String>,
    /// Minimum price (in cents)
    pub min_price: Option<i64>,
    /// Maximum price (in cents)
    pub max_price: Option<i64>,
    /// Only show products in stock
    pub in_stock: Option<bool>,
    /// Filter by tag
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

/// Stock adjustment request
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct StockAdjustment {
    /// Quantity to add (positive) or remove (negative)
    pub quantity: i32,
    /// Reason for adjustment
    #[validate(length(min = 1, max = 500))]
    pub reason: String,
}

/// Stock reservation request
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct StockReservation {
    /// Quantity to reserve
    #[validate(range(min = 1))]
    pub quantity: i32,
    /// Order ID for tracking
    pub order_id: String,
    /// TTL in seconds for the reservation
    #[serde(default = "default_reservation_ttl")]
    pub ttl_seconds: u32,
}

/// Stock reservation response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReservationResult {
    /// Unique reservation ID
    pub reservation_id: String,
    /// Whether the reservation was successful
    pub success: bool,
    /// Message (error or success details)
    pub message: String,
}

fn default_limit() -> i64 {
    50
}

fn default_reservation_ttl() -> u32 {
    900 // 15 minutes
}

impl Product {
    /// Create a new product from CreateProduct DTO
    pub fn new(input: CreateProduct) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: input.name,
            description: input.description,
            price: input.price,
            display_price: Some(input.price as f64 / 100.0),
            stock: input.stock,
            reserved_stock: 0,
            category: input.category,
            status: input.status,
            images: input.images,
            sku: input.sku,
            barcode: input.barcode,
            brand: input.brand,
            weight: input.weight,
            tags: input.tags,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Apply updates from UpdateProduct DTO
    pub fn apply_update(&mut self, update: UpdateProduct) {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(description) = update.description {
            self.description = description;
        }
        if let Some(price) = update.price {
            self.price = price;
            self.display_price = Some(price as f64 / 100.0);
        }
        if let Some(stock) = update.stock {
            self.stock = stock;
        }
        if let Some(category) = update.category {
            self.category = category;
        }
        if let Some(status) = update.status {
            self.status = status;
        }
        if let Some(images) = update.images {
            self.images = images;
        }
        if let Some(sku) = update.sku {
            self.sku = Some(sku);
        }
        if let Some(barcode) = update.barcode {
            self.barcode = Some(barcode);
        }
        if let Some(brand) = update.brand {
            self.brand = Some(brand);
        }
        if let Some(weight) = update.weight {
            self.weight = Some(weight);
        }
        if let Some(tags) = update.tags {
            self.tags = tags;
        }
        if let Some(metadata) = update.metadata {
            self.metadata = metadata;
        }
        self.updated_at = Utc::now();
    }

    /// Get available stock (total stock minus reserved)
    pub fn available_stock(&self) -> i32 {
        self.stock - self.reserved_stock
    }

    /// Check if product is in stock
    pub fn is_in_stock(&self) -> bool {
        self.available_stock() > 0
    }

    /// Adjust stock quantity
    pub fn adjust_stock(&mut self, quantity: i32) -> bool {
        let new_stock = self.stock + quantity;
        if new_stock >= 0 {
            self.stock = new_stock;
            self.updated_at = Utc::now();

            // Update status if out of stock
            if self.stock == 0 && self.status == ProductStatus::Active {
                self.status = ProductStatus::OutOfStock;
            } else if self.stock > 0 && self.status == ProductStatus::OutOfStock {
                self.status = ProductStatus::Active;
            }

            true
        } else {
            false
        }
    }

    /// Reserve stock for an order
    pub fn reserve_stock(&mut self, quantity: i32) -> bool {
        if self.available_stock() >= quantity {
            self.reserved_stock += quantity;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Release reserved stock
    pub fn release_stock(&mut self, quantity: i32) {
        self.reserved_stock = (self.reserved_stock - quantity).max(0);
        self.updated_at = Utc::now();
    }

    /// Commit reserved stock (after order completion)
    pub fn commit_reserved_stock(&mut self, quantity: i32) -> bool {
        if self.reserved_stock >= quantity {
            self.reserved_stock -= quantity;
            self.stock -= quantity;
            self.updated_at = Utc::now();

            if self.stock == 0 {
                self.status = ProductStatus::OutOfStock;
            }

            true
        } else {
            false
        }
    }
}

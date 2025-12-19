use chrono::{DateTime, Utc};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Cloud provider enumeration
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
    DeriveActiveEnum,
    EnumIter,
    ToSchema,
    TS,
    Hash,
)]
#[ts(export)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cloud_provider")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CloudProvider {
    #[default]
    #[sea_orm(string_value = "aws")]
    Aws,
    #[sea_orm(string_value = "azure")]
    Azure,
    #[sea_orm(string_value = "gcp")]
    Gcp,
}

/// Resource type enumeration
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
    DeriveActiveEnum,
    EnumIter,
    ToSchema,
    TS,
    Hash,
)]
#[ts(export)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "resource_type")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ResourceType {
    #[default]
    #[sea_orm(string_value = "compute")]
    Compute,
    #[sea_orm(string_value = "storage")]
    Storage,
    #[sea_orm(string_value = "database")]
    Database,
    #[sea_orm(string_value = "network")]
    Network,
    #[sea_orm(string_value = "serverless")]
    Serverless,
    #[sea_orm(string_value = "analytics")]
    Analytics,
    #[sea_orm(string_value = "kubernetes")]
    Kubernetes,
    #[sea_orm(string_value = "other")]
    Other,
}

/// Pricing unit enumeration
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
    DeriveActiveEnum,
    EnumIter,
    ToSchema,
    TS,
    Hash,
)]
#[ts(export)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "pricing_unit")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PricingUnit {
    #[default]
    #[sea_orm(string_value = "hour")]
    Hour,
    #[sea_orm(string_value = "month")]
    Month,
    #[sea_orm(string_value = "gb")]
    Gb,
    #[sea_orm(string_value = "gb_hour")]
    GbHour,
    #[sea_orm(string_value = "gb_month")]
    GbMonth,
    #[sea_orm(string_value = "request")]
    Request,
    #[sea_orm(string_value = "million_requests")]
    MillionRequests,
    #[sea_orm(string_value = "second")]
    Second,
    #[sea_orm(string_value = "unit")]
    Unit,
}

/// Currency enumeration
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
    DeriveActiveEnum,
    EnumIter,
    ToSchema,
    TS,
    Hash,
)]
#[ts(export)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "currency")]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Currency {
    #[default]
    #[sea_orm(string_value = "USD")]
    Usd,
    #[sea_orm(string_value = "EUR")]
    Eur,
    #[sea_orm(string_value = "GBP")]
    Gbp,
}

/// Money representation with precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS, Hash)]
#[ts(export)]
pub struct Money {
    /// Amount in the smallest currency unit (cents for USD)
    pub amount: i64,
    /// Currency type
    pub currency: Currency,
    /// Number of decimal places (2 for cents)
    pub decimal_places: i32,
}

impl Money {
    /// Create a new Money value
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self {
            amount,
            currency,
            decimal_places: 2,
        }
    }

    /// Create Money from a decimal value (e.g., 1.99 USD)
    pub fn from_decimal(value: f64, currency: Currency) -> Self {
        Self {
            amount: (value * 100.0).round() as i64,
            currency,
            decimal_places: 2,
        }
    }

    /// Convert to decimal value
    pub fn to_decimal(&self) -> f64 {
        self.amount as f64 / 10f64.powi(self.decimal_places)
    }
}

impl Default for Money {
    fn default() -> Self {
        Self {
            amount: 0,
            currency: Currency::Usd,
            decimal_places: 2,
        }
    }
}

/// Price entry representing a single pricing record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct PriceEntry {
    /// Unique identifier
    #[ts(as = "String")]
    pub id: Uuid,
    /// Cloud provider
    pub provider: CloudProvider,
    /// Resource type
    pub resource_type: ResourceType,
    /// Provider-specific SKU
    pub sku: String,
    /// Service name (e.g., "Amazon EC2", "Azure VMs")
    pub service_name: String,
    /// Product family (e.g., "Compute Instance")
    pub product_family: String,
    /// Instance type (e.g., "t3.medium", "Standard_D2s_v3")
    pub instance_type: Option<String>,
    /// Region code
    pub region: String,
    /// Unit price
    pub unit_price: Money,
    /// Pricing unit
    pub pricing_unit: PricingUnit,
    /// Description
    pub description: String,
    /// Additional attributes (vCPU, memory, etc.)
    pub attributes: HashMap<String, String>,
    /// When this price became effective
    #[ts(as = "String")]
    pub effective_date: DateTime<Utc>,
    /// When this price expires (null if current)
    #[ts(as = "Option<String>")]
    pub expiration_date: Option<DateTime<Utc>>,
    /// When we fetched this price
    #[ts(as = "String")]
    pub collected_at: DateTime<Utc>,
    /// Creation timestamp
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[ts(as = "String")]
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new price entry
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, TS)]
#[ts(export)]
pub struct CreatePriceEntry {
    pub provider: CloudProvider,
    pub resource_type: ResourceType,
    #[validate(length(min = 1, max = 255))]
    pub sku: String,
    #[validate(length(min = 1, max = 255))]
    pub service_name: String,
    #[validate(length(max = 255))]
    pub product_family: String,
    pub instance_type: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub region: String,
    pub unit_price: Money,
    pub pricing_unit: PricingUnit,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[ts(as = "String")]
    pub effective_date: DateTime<Utc>,
    #[ts(as = "Option<String>")]
    pub expiration_date: Option<DateTime<Utc>>,
}

/// DTO for updating an existing price entry
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, Default, TS)]
#[ts(export)]
pub struct UpdatePriceEntry {
    pub unit_price: Option<Money>,
    pub description: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
    #[ts(as = "Option<String>")]
    pub expiration_date: Option<Option<DateTime<Utc>>>,
}

/// Query filters for listing prices
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, Default)]
pub struct PriceFilter {
    pub provider: Option<CloudProvider>,
    pub resource_type: Option<ResourceType>,
    /// Comma-separated list of regions (e.g., "us-east-1,eu-west-1")
    pub regions: Option<String>,
    pub service_name: Option<String>,
    pub instance_type: Option<String>,
    pub sku: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

impl PriceFilter {
    /// Parse regions from comma-separated string to Vec
    pub fn regions_vec(&self) -> Option<Vec<String>> {
        self.regions.as_ref().map(|r| {
            r.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }
}

fn default_limit() -> usize {
    50
}

/// Price comparison result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct PriceComparison {
    /// Comparison key (e.g., "4vcpu-8gb-compute")
    pub comparison_key: String,
    /// Prices from each provider
    pub provider_prices: Vec<ProviderPrice>,
    /// Cheapest option
    pub cheapest: Option<ProviderPrice>,
    /// Potential savings vs. most expensive
    pub potential_savings: Option<Money>,
}

/// Provider price for comparison
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ProviderPrice {
    pub provider: CloudProvider,
    pub price: PriceEntry,
    /// Estimated monthly cost
    pub monthly_estimate: Money,
}

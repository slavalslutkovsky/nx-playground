use crate::models::{CloudProvider, Currency, PricingUnit, ResourceType};
use core_proc_macros::SeaOrmResource;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

/// Sea-ORM Entity for cloud_prices table
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource)]
#[sea_orm(table_name = "cloud_prices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub provider: CloudProvider,
    pub resource_type: ResourceType,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub sku: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub service_name: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub product_family: String,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub instance_type: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub region: String,
    /// Unit price amount in smallest currency unit (cents)
    pub unit_price_amount: i64,
    pub unit_price_currency: Currency,
    pub pricing_unit: PricingUnit,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    /// JSON-encoded attributes
    #[sea_orm(column_type = "JsonBinary")]
    pub attributes: serde_json::Value,
    pub effective_date: DateTimeWithTimeZone,
    pub expiration_date: Option<DateTimeWithTimeZone>,
    pub collected_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Conversion from Sea-ORM Model to domain PriceEntry
impl From<Model> for crate::models::PriceEntry {
    fn from(model: Model) -> Self {
        let attributes: std::collections::HashMap<String, String> =
            serde_json::from_value(model.attributes).unwrap_or_default();

        Self {
            id: model.id,
            provider: model.provider,
            resource_type: model.resource_type,
            sku: model.sku,
            service_name: model.service_name,
            product_family: model.product_family,
            instance_type: model.instance_type,
            region: model.region,
            unit_price: crate::models::Money {
                amount: model.unit_price_amount,
                currency: model.unit_price_currency,
                decimal_places: 2,
            },
            pricing_unit: model.pricing_unit,
            description: model.description,
            attributes,
            effective_date: model.effective_date.into(),
            expiration_date: model.expiration_date.map(Into::into),
            collected_at: model.collected_at.into(),
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

// Conversion from domain CreatePriceEntry to Sea-ORM ActiveModel
impl From<crate::models::CreatePriceEntry> for ActiveModel {
    fn from(input: crate::models::CreatePriceEntry) -> Self {
        let now = chrono::Utc::now();
        ActiveModel {
            id: Set(Uuid::now_v7()),
            provider: Set(input.provider),
            resource_type: Set(input.resource_type),
            sku: Set(input.sku),
            service_name: Set(input.service_name),
            product_family: Set(input.product_family),
            instance_type: Set(input.instance_type),
            region: Set(input.region),
            unit_price_amount: Set(input.unit_price.amount),
            unit_price_currency: Set(input.unit_price.currency),
            pricing_unit: Set(input.pricing_unit),
            description: Set(input.description),
            attributes: Set(serde_json::to_value(input.attributes).unwrap_or_default()),
            effective_date: Set(input.effective_date.into()),
            expiration_date: Set(input.expiration_date.map(Into::into)),
            collected_at: Set(now.into()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
    }
}

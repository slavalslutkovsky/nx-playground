//! Conversions between protobuf types and domain types

use crate::models::{
    CloudProvider, CreatePriceEntry, Currency, Money, PriceEntry, PricingUnit, ResourceType,
};
use protos::cloud::v1 as proto;

// ============================================================================
// CloudProvider conversions
// ============================================================================

impl From<proto::CloudProvider> for CloudProvider {
    fn from(proto: proto::CloudProvider) -> Self {
        match proto {
            proto::CloudProvider::Aws => CloudProvider::Aws,
            proto::CloudProvider::Azure => CloudProvider::Azure,
            proto::CloudProvider::Gcp => CloudProvider::Gcp,
            proto::CloudProvider::Unspecified => CloudProvider::Aws, // Default
        }
    }
}

impl From<CloudProvider> for proto::CloudProvider {
    fn from(domain: CloudProvider) -> Self {
        match domain {
            CloudProvider::Aws => proto::CloudProvider::Aws,
            CloudProvider::Azure => proto::CloudProvider::Azure,
            CloudProvider::Gcp => proto::CloudProvider::Gcp,
        }
    }
}

impl From<i32> for CloudProvider {
    fn from(value: i32) -> Self {
        proto::CloudProvider::try_from(value)
            .unwrap_or(proto::CloudProvider::Aws)
            .into()
    }
}

// ============================================================================
// ResourceType conversions
// ============================================================================

impl From<proto::ResourceType> for ResourceType {
    fn from(proto: proto::ResourceType) -> Self {
        match proto {
            proto::ResourceType::Compute => ResourceType::Compute,
            proto::ResourceType::Storage => ResourceType::Storage,
            proto::ResourceType::Database => ResourceType::Database,
            proto::ResourceType::Network => ResourceType::Network,
            proto::ResourceType::Serverless => ResourceType::Serverless,
            proto::ResourceType::Analytics => ResourceType::Analytics,
            proto::ResourceType::Kubernetes => ResourceType::Kubernetes,
            proto::ResourceType::Other | proto::ResourceType::Unspecified => ResourceType::Other,
        }
    }
}

impl From<ResourceType> for proto::ResourceType {
    fn from(domain: ResourceType) -> Self {
        match domain {
            ResourceType::Compute => proto::ResourceType::Compute,
            ResourceType::Storage => proto::ResourceType::Storage,
            ResourceType::Database => proto::ResourceType::Database,
            ResourceType::Network => proto::ResourceType::Network,
            ResourceType::Serverless => proto::ResourceType::Serverless,
            ResourceType::Analytics => proto::ResourceType::Analytics,
            ResourceType::Kubernetes => proto::ResourceType::Kubernetes,
            ResourceType::Other => proto::ResourceType::Other,
        }
    }
}

impl From<i32> for ResourceType {
    fn from(value: i32) -> Self {
        proto::ResourceType::try_from(value)
            .unwrap_or(proto::ResourceType::Other)
            .into()
    }
}

// ============================================================================
// PricingUnit conversions
// ============================================================================

impl From<proto::PricingUnit> for PricingUnit {
    fn from(proto: proto::PricingUnit) -> Self {
        match proto {
            proto::PricingUnit::Hour => PricingUnit::Hour,
            proto::PricingUnit::Month => PricingUnit::Month,
            proto::PricingUnit::Gb => PricingUnit::Gb,
            proto::PricingUnit::GbHour => PricingUnit::GbHour,
            proto::PricingUnit::GbMonth => PricingUnit::GbMonth,
            proto::PricingUnit::Request => PricingUnit::Request,
            proto::PricingUnit::MillionRequests => PricingUnit::MillionRequests,
            proto::PricingUnit::Second => PricingUnit::Second,
            proto::PricingUnit::Unit | proto::PricingUnit::Unspecified => PricingUnit::Unit,
        }
    }
}

impl From<PricingUnit> for proto::PricingUnit {
    fn from(domain: PricingUnit) -> Self {
        match domain {
            PricingUnit::Hour => proto::PricingUnit::Hour,
            PricingUnit::Month => proto::PricingUnit::Month,
            PricingUnit::Gb => proto::PricingUnit::Gb,
            PricingUnit::GbHour => proto::PricingUnit::GbHour,
            PricingUnit::GbMonth => proto::PricingUnit::GbMonth,
            PricingUnit::Request => proto::PricingUnit::Request,
            PricingUnit::MillionRequests => proto::PricingUnit::MillionRequests,
            PricingUnit::Second => proto::PricingUnit::Second,
            PricingUnit::Unit => proto::PricingUnit::Unit,
        }
    }
}

impl From<i32> for PricingUnit {
    fn from(value: i32) -> Self {
        proto::PricingUnit::try_from(value)
            .unwrap_or(proto::PricingUnit::Unit)
            .into()
    }
}

// ============================================================================
// Currency conversions
// ============================================================================

impl From<proto::Currency> for Currency {
    fn from(proto: proto::Currency) -> Self {
        match proto {
            proto::Currency::Usd | proto::Currency::Unspecified => Currency::Usd,
            proto::Currency::Eur => Currency::Eur,
            proto::Currency::Gbp => Currency::Gbp,
        }
    }
}

impl From<Currency> for proto::Currency {
    fn from(domain: Currency) -> Self {
        match domain {
            Currency::Usd => proto::Currency::Usd,
            Currency::Eur => proto::Currency::Eur,
            Currency::Gbp => proto::Currency::Gbp,
        }
    }
}

impl From<i32> for Currency {
    fn from(value: i32) -> Self {
        proto::Currency::try_from(value)
            .unwrap_or(proto::Currency::Usd)
            .into()
    }
}

// ============================================================================
// Money conversions
// ============================================================================

impl From<proto::Money> for Money {
    fn from(proto: proto::Money) -> Self {
        Money {
            amount: proto.amount,
            currency: proto.currency.into(),
            decimal_places: proto.decimal_places,
        }
    }
}

impl From<Money> for proto::Money {
    fn from(domain: Money) -> Self {
        proto::Money {
            amount: domain.amount,
            currency: proto::Currency::from(domain.currency) as i32,
            decimal_places: domain.decimal_places,
        }
    }
}

// ============================================================================
// PriceEntry conversions
// ============================================================================

impl From<PriceEntry> for proto::PriceEntry {
    fn from(domain: PriceEntry) -> Self {
        proto::PriceEntry {
            id: domain.id.as_bytes().to_vec(),
            provider: proto::CloudProvider::from(domain.provider) as i32,
            resource_type: proto::ResourceType::from(domain.resource_type) as i32,
            sku: domain.sku,
            service_name: domain.service_name,
            product_family: domain.product_family,
            instance_type: domain.instance_type.unwrap_or_default(),
            region: domain.region,
            unit_price: Some(domain.unit_price.into()),
            pricing_unit: proto::PricingUnit::from(domain.pricing_unit) as i32,
            description: domain.description,
            attributes: domain.attributes,
            effective_date: domain.effective_date.timestamp(),
            expiration_date: domain.expiration_date.map(|dt| dt.timestamp()),
            collected_at: domain.collected_at.timestamp(),
        }
    }
}

impl TryFrom<proto::PriceEntry> for PriceEntry {
    type Error = String;

    fn try_from(proto: proto::PriceEntry) -> Result<Self, Self::Error> {
        let id = uuid::Uuid::from_slice(&proto.id)
            .map_err(|e| format!("Invalid UUID: {}", e))?;

        let unit_price = proto
            .unit_price
            .map(Into::into)
            .ok_or("Missing unit_price")?;

        Ok(PriceEntry {
            id,
            provider: proto.provider.into(),
            resource_type: proto.resource_type.into(),
            sku: proto.sku,
            service_name: proto.service_name,
            product_family: proto.product_family,
            instance_type: if proto.instance_type.is_empty() {
                None
            } else {
                Some(proto.instance_type)
            },
            region: proto.region,
            unit_price,
            pricing_unit: proto.pricing_unit.into(),
            description: proto.description,
            attributes: proto.attributes,
            effective_date: chrono::DateTime::from_timestamp(proto.effective_date, 0)
                .ok_or("Invalid effective_date")?
                .into(),
            expiration_date: proto
                .expiration_date
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(Into::into),
            collected_at: chrono::DateTime::from_timestamp(proto.collected_at, 0)
                .ok_or("Invalid collected_at")?
                .into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

// ============================================================================
// CreatePriceEntry from proto
// ============================================================================

impl TryFrom<proto::PriceEntry> for CreatePriceEntry {
    type Error = String;

    fn try_from(proto: proto::PriceEntry) -> Result<Self, Self::Error> {
        let unit_price = proto
            .unit_price
            .map(Into::into)
            .ok_or("Missing unit_price")?;

        Ok(CreatePriceEntry {
            provider: proto.provider.into(),
            resource_type: proto.resource_type.into(),
            sku: proto.sku,
            service_name: proto.service_name,
            product_family: proto.product_family,
            instance_type: if proto.instance_type.is_empty() {
                None
            } else {
                Some(proto.instance_type)
            },
            region: proto.region,
            unit_price,
            pricing_unit: proto.pricing_unit.into(),
            description: proto.description,
            attributes: proto.attributes,
            effective_date: chrono::DateTime::from_timestamp(proto.effective_date, 0)
                .ok_or("Invalid effective_date")?
                .into(),
            expiration_date: proto
                .expiration_date
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(Into::into),
        })
    }
}

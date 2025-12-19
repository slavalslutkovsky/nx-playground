use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::entity::{ActiveModel, Column, Entity};
use crate::error::{PricingError, PricingResult};
use crate::models::{CloudProvider, CreatePriceEntry, PriceEntry, PriceFilter, UpdatePriceEntry};
use crate::repository::PricingRepository;

/// PostgreSQL implementation of PricingRepository
#[derive(Clone)]
pub struct PgPricingRepository {
    db: DatabaseConnection,
}

impl PgPricingRepository {
    /// Create a new PostgreSQL pricing repository
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PricingRepository for PgPricingRepository {
    async fn create(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry> {
        let model: ActiveModel = input.into();
        let result = model.insert(&self.db).await?.into();
        Ok(result)
    }

    async fn create_many(&self, inputs: Vec<CreatePriceEntry>) -> PricingResult<Vec<PriceEntry>> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let model: ActiveModel = input.into();
            let result = model.insert(&self.db).await?;
            results.push(result.into());
        }
        Ok(results)
    }

    async fn get_by_id(&self, id: Uuid) -> PricingResult<Option<PriceEntry>> {
        let result = Entity::find_by_id(id).one(&self.db).await?.map(Into::into);
        Ok(result)
    }

    async fn get_by_sku(
        &self,
        sku: &str,
        provider: CloudProvider,
        region: &str,
    ) -> PricingResult<Option<PriceEntry>> {
        let result = Entity::find()
            .filter(Column::Sku.eq(sku))
            .filter(Column::Provider.eq(provider))
            .filter(Column::Region.eq(region))
            .one(&self.db)
            .await?
          .map(Into::into);
        Ok(result)
    }

    async fn list(&self, filter: PriceFilter) -> PricingResult<Vec<PriceEntry>> {
        let mut query = Entity::find();

        if let Some(provider) = filter.provider {
            query = query.filter(Column::Provider.eq(provider));
        }

        if let Some(resource_type) = filter.resource_type {
            query = query.filter(Column::ResourceType.eq(resource_type));
        }

        if let Some(regions) = filter.regions_vec() {
            if !regions.is_empty() {
                query = query.filter(Column::Region.is_in(regions));
            }
        }

        if let Some(service_name) = filter.service_name {
            query = query.filter(Column::ServiceName.contains(&service_name));
        }

        if let Some(instance_type) = filter.instance_type {
            query = query.filter(Column::InstanceType.eq(instance_type));
        }

        if let Some(sku) = filter.sku {
            query = query.filter(Column::Sku.eq(sku));
        }

        let results = query
            .order_by_desc(Column::UpdatedAt)
            .offset(filter.offset as u64)
            .limit(filter.limit as u64)
            .all(&self.db)
            .await?.into_iter().map(Into::into).collect();

        Ok(results)
    }

    async fn update(&self, id: Uuid, input: UpdatePriceEntry) -> PricingResult<PriceEntry> {
        let existing = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| PricingError::NotFound(id.to_string()))?;

        let mut model: ActiveModel = existing.into();

        if let Some(unit_price) = input.unit_price {
            model.unit_price_amount = Set(unit_price.amount);
            model.unit_price_currency = Set(unit_price.currency);
        }

        if let Some(description) = input.description {
            model.description = Set(description);
        }

        if let Some(attributes) = input.attributes {
            model.attributes = Set(serde_json::to_value(attributes).unwrap_or_default());
        }

        if let Some(expiration_date) = input.expiration_date {
            model.expiration_date = Set(expiration_date.map(Into::into));
        }

        model.updated_at = Set(chrono::Utc::now().into());

        let result = model.update(&self.db).await?.into();
        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> PricingResult<bool> {
        let result = Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(result.rows_affected > 0)
    }

    async fn upsert(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry> {
        // Check if entry exists
        if let Some(existing) =
            self.get_by_sku(&input.sku, input.provider, &input.region).await?
        {
            // Update existing entry
            let update = UpdatePriceEntry {
                unit_price: Some(input.unit_price),
                description: Some(input.description),
                attributes: Some(input.attributes),
                expiration_date: Some(input.expiration_date),
            };
            self.update(existing.id, update).await
        } else {
            // Create a new entry
            self.create(input).await
        }
    }

    async fn count(&self) -> PricingResult<usize> {
        let count = Entity::find().count(&self.db).await?;
        Ok(count as usize)
    }

    async fn count_by_provider(&self, provider: CloudProvider) -> PricingResult<usize> {
        let count = Entity::find()
            .filter(Column::Provider.eq(provider))
            .count(&self.db)
            .await?;
        Ok(count as usize)
    }

    async fn get_regions_for_provider(&self, provider: CloudProvider) -> PricingResult<Vec<String>> {
        let results = Entity::find()
            .filter(Column::Provider.eq(provider))
            .select_only()
            .column(Column::Region)
            .distinct()
            .into_tuple::<String>()
            .all(&self.db)
            .await?;
        Ok(results)
    }

    async fn delete_expired(&self) -> PricingResult<usize> {
        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        let result = Entity::delete_many()
            .filter(Column::ExpirationDate.lt(now))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected as usize)
    }
}

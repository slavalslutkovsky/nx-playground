use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add 'kubernetes' to existing resource_type enum
        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE resource_type ADD VALUE IF NOT EXISTS 'kubernetes'")
            .await?;

        // Create pricing_unit enum (new)
        manager
            .create_type(
                Type::create()
                    .as_enum(PricingUnitEnum::Enum)
                    .values([
                        PricingUnitEnum::Hour,
                        PricingUnitEnum::Month,
                        PricingUnitEnum::Gb,
                        PricingUnitEnum::GbHour,
                        PricingUnitEnum::GbMonth,
                        PricingUnitEnum::Request,
                        PricingUnitEnum::MillionRequests,
                        PricingUnitEnum::Second,
                        PricingUnitEnum::Unit,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create currency enum (new)
        manager
            .create_type(
                Type::create()
                    .as_enum(CurrencyEnum::Enum)
                    .values([
                        CurrencyEnum::Usd,
                        CurrencyEnum::Eur,
                        CurrencyEnum::Gbp,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create cloud_prices table using existing cloud_provider and resource_type enums
        manager
            .create_table(
                Table::create()
                    .table(CloudPrices::Table)
                    .if_not_exists()
                    .col(pk_uuid(CloudPrices::Id))
                    .col(
                        ColumnDef::new(CloudPrices::Provider)
                            .enumeration(
                                CloudProviderEnum::Enum,
                                [
                                    CloudProviderEnum::Aws,
                                    CloudProviderEnum::Azure,
                                    CloudProviderEnum::Gcp,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CloudPrices::ResourceType)
                            .enumeration(
                                ResourceTypeEnum::Enum,
                                [
                                    ResourceTypeEnum::Compute,
                                    ResourceTypeEnum::Storage,
                                    ResourceTypeEnum::Database,
                                    ResourceTypeEnum::Network,
                                    ResourceTypeEnum::Serverless,
                                    ResourceTypeEnum::Analytics,
                                    ResourceTypeEnum::Kubernetes,
                                    ResourceTypeEnum::Other,
                                ],
                            )
                            .not_null(),
                    )
                    .col(string_len(CloudPrices::Sku, 255).not_null())
                    .col(string_len(CloudPrices::ServiceName, 255).not_null())
                    .col(string_len(CloudPrices::ProductFamily, 255).not_null())
                    .col(string_len_null(CloudPrices::InstanceType, 255))
                    .col(string_len(CloudPrices::Region, 100).not_null())
                    .col(big_integer(CloudPrices::UnitPriceAmount).not_null())
                    .col(
                        ColumnDef::new(CloudPrices::UnitPriceCurrency)
                            .enumeration(
                                CurrencyEnum::Enum,
                                [CurrencyEnum::Usd, CurrencyEnum::Eur, CurrencyEnum::Gbp],
                            )
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(CloudPrices::PricingUnit)
                            .enumeration(
                                PricingUnitEnum::Enum,
                                [
                                    PricingUnitEnum::Hour,
                                    PricingUnitEnum::Month,
                                    PricingUnitEnum::Gb,
                                    PricingUnitEnum::GbHour,
                                    PricingUnitEnum::GbMonth,
                                    PricingUnitEnum::Request,
                                    PricingUnitEnum::MillionRequests,
                                    PricingUnitEnum::Second,
                                    PricingUnitEnum::Unit,
                                ],
                            )
                            .not_null(),
                    )
                    .col(text(CloudPrices::Description).default(""))
                    .col(json_binary(CloudPrices::Attributes).not_null().default("{}"))
                    .col(timestamp_with_time_zone(CloudPrices::EffectiveDate).not_null())
                    .col(timestamp_with_time_zone_null(CloudPrices::ExpirationDate))
                    .col(
                        timestamp_with_time_zone(CloudPrices::CollectedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(CloudPrices::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(CloudPrices::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on SKU + provider + region (for upsert)
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_sku_provider_region")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::Sku)
                    .col(CloudPrices::Provider)
                    .col(CloudPrices::Region)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create indexes for common queries
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_provider")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::Provider)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_resource_type")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::ResourceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_region")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::Region)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_service_name")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::ServiceName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_instance_type")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::InstanceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_expiration_date")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::ExpirationDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_prices_updated_at")
                    .table(CloudPrices::Table)
                    .col(CloudPrices::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER cloud_prices_touch_updated_at
                    BEFORE UPDATE ON cloud_prices
                    FOR EACH ROW
                    EXECUTE FUNCTION util.touch_updated_at()
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS cloud_prices_touch_updated_at ON cloud_prices")
            .await?;

        manager
            .drop_table(Table::drop().table(CloudPrices::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(CurrencyEnum::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(PricingUnitEnum::Enum).to_owned())
            .await?;

        // Note: We don't remove 'kubernetes' from resource_type as it may be in use
        // and ALTER TYPE doesn't support removing values in PostgreSQL

        Ok(())
    }
}

#[derive(DeriveIden)]
enum CloudPrices {
    Table,
    Id,
    Provider,
    ResourceType,
    Sku,
    ServiceName,
    ProductFamily,
    InstanceType,
    Region,
    UnitPriceAmount,
    UnitPriceCurrency,
    PricingUnit,
    Description,
    Attributes,
    EffectiveDate,
    ExpirationDate,
    CollectedAt,
    CreatedAt,
    UpdatedAt,
}

// Reference to existing cloud_provider enum (created in projects migration)
#[derive(DeriveIden)]
enum CloudProviderEnum {
    #[sea_orm(iden = "cloud_provider")]
    Enum,
    #[sea_orm(iden = "aws")]
    Aws,
    #[sea_orm(iden = "azure")]
    Azure,
    #[sea_orm(iden = "gcp")]
    Gcp,
    #[sea_orm(iden = "cloudflare")]
    Cloudflare,
}

// Reference to existing resource_type enum (created in cloud_resources migration)
// Added 'kubernetes' value in this migration
#[derive(DeriveIden)]
enum ResourceTypeEnum {
    #[sea_orm(iden = "resource_type")]
    Enum,
    #[sea_orm(iden = "compute")]
    Compute,
    #[sea_orm(iden = "storage")]
    Storage,
    #[sea_orm(iden = "database")]
    Database,
    #[sea_orm(iden = "network")]
    Network,
    #[sea_orm(iden = "serverless")]
    Serverless,
    #[sea_orm(iden = "analytics")]
    Analytics,
    #[sea_orm(iden = "kubernetes")]
    Kubernetes,
    #[sea_orm(iden = "other")]
    Other,
}

#[derive(DeriveIden)]
enum PricingUnitEnum {
    #[sea_orm(iden = "pricing_unit")]
    Enum,
    #[sea_orm(iden = "hour")]
    Hour,
    #[sea_orm(iden = "month")]
    Month,
    #[sea_orm(iden = "gb")]
    Gb,
    #[sea_orm(iden = "gb_hour")]
    GbHour,
    #[sea_orm(iden = "gb_month")]
    GbMonth,
    #[sea_orm(iden = "request")]
    Request,
    #[sea_orm(iden = "million_requests")]
    MillionRequests,
    #[sea_orm(iden = "second")]
    Second,
    #[sea_orm(iden = "unit")]
    Unit,
}

#[derive(DeriveIden)]
enum CurrencyEnum {
    #[sea_orm(iden = "currency")]
    Enum,
    #[sea_orm(iden = "USD")]
    Usd,
    #[sea_orm(iden = "EUR")]
    Eur,
    #[sea_orm(iden = "GBP")]
    Gbp,
}

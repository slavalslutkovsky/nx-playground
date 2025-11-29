use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create resource_type enum
        manager
            .create_type(
                Type::create()
                    .as_enum(ResourceType::Enum)
                    .values([
                        ResourceType::Compute,
                        ResourceType::Storage,
                        ResourceType::Database,
                        ResourceType::Network,
                        ResourceType::Serverless,
                        ResourceType::Analytics,
                        ResourceType::Other,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create resource_status enum
        manager
            .create_type(
                Type::create()
                    .as_enum(ResourceStatus::Enum)
                    .values([
                        ResourceStatus::Creating,
                        ResourceStatus::Active,
                        ResourceStatus::Updating,
                        ResourceStatus::Deleting,
                        ResourceStatus::Deleted,
                        ResourceStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create cloud_resources table
        manager
            .create_table(
                Table::create()
                    .table(CloudResources::Table)
                    .if_not_exists()
                    .col(pk_uuid(CloudResources::Id))
                    .col(uuid(CloudResources::ProjectId))
                    .col(string(CloudResources::Name))
                    .col(
                        ColumnDef::new(CloudResources::ResourceType)
                            .enumeration(
                                ResourceType::Enum,
                                [
                                    ResourceType::Compute,
                                    ResourceType::Storage,
                                    ResourceType::Database,
                                    ResourceType::Network,
                                    ResourceType::Serverless,
                                    ResourceType::Analytics,
                                    ResourceType::Other,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CloudResources::Status)
                            .enumeration(
                                ResourceStatus::Enum,
                                [
                                    ResourceStatus::Creating,
                                    ResourceStatus::Active,
                                    ResourceStatus::Updating,
                                    ResourceStatus::Deleting,
                                    ResourceStatus::Deleted,
                                    ResourceStatus::Failed,
                                ],
                            )
                            .not_null()
                            .default("creating"),
                    )
                    .col(string(CloudResources::Region))
                    .col(json(CloudResources::Configuration).default("{}"))
                    .col(double_null(CloudResources::CostPerHour))
                    .col(double_null(CloudResources::MonthlyCostEstimate))
                    .col(json(CloudResources::Tags).default("[]"))
                    .col(boolean(CloudResources::Enabled).default(true))
                    .col(
                        timestamp_with_time_zone(CloudResources::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(CloudResources::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(CloudResources::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cloud_resources_project_id")
                            .from(CloudResources::Table, CloudResources::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_resources_project_id")
                    .table(CloudResources::Table)
                    .col(CloudResources::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_resources_resource_type")
                    .table(CloudResources::Table)
                    .col(CloudResources::ResourceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_resources_status")
                    .table(CloudResources::Table)
                    .col(CloudResources::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_resources_region")
                    .table(CloudResources::Table)
                    .col(CloudResources::Region)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_resources_created_at")
                    .table(CloudResources::Table)
                    .col(CloudResources::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Unique constraint
        manager
            .create_index(
                Index::create()
                    .name("unique_resource_name_per_project")
                    .table(CloudResources::Table)
                    .col(CloudResources::ProjectId)
                    .col(CloudResources::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CloudResources::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ResourceStatus::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ResourceType::Enum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum CloudResources {
    Table,
    Id,
    ProjectId,
    Name,
    ResourceType,
    Status,
    Region,
    Configuration,
    CostPerHour,
    MonthlyCostEstimate,
    Tags,
    Enabled,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ResourceType {
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
    #[sea_orm(iden = "other")]
    Other,
}

#[derive(DeriveIden)]
enum ResourceStatus {
    #[sea_orm(iden = "resource_status")]
    Enum,
    #[sea_orm(iden = "creating")]
    Creating,
    #[sea_orm(iden = "active")]
    Active,
    #[sea_orm(iden = "updating")]
    Updating,
    #[sea_orm(iden = "deleting")]
    Deleting,
    #[sea_orm(iden = "deleted")]
    Deleted,
    #[sea_orm(iden = "failed")]
    Failed,
}

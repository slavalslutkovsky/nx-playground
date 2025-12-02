use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create cloud_provider enum
        manager
            .create_type(
                Type::create()
                    .as_enum(CloudProvider::Enum)
                    .values([CloudProvider::Aws, CloudProvider::Gcp, CloudProvider::Azure])
                    .to_owned(),
            )
            .await?;

        // Create project_status enum
        manager
            .create_type(
                Type::create()
                    .as_enum(ProjectStatus::Enum)
                    .values([
                        ProjectStatus::Provisioning,
                        ProjectStatus::Active,
                        ProjectStatus::Suspended,
                        ProjectStatus::Deleting,
                        ProjectStatus::Archived,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create environment enum
        manager
            .create_type(
                Type::create()
                    .as_enum(Environment::Enum)
                    .values([
                        Environment::Development,
                        Environment::Staging,
                        Environment::Production,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create projects table
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(pk_uuid(Projects::Id))
                    .col(string(Projects::Name))
                    .col(uuid(Projects::UserId))
                    .col(string(Projects::Description).default(""))
                    .col(
                        ColumnDef::new(Projects::CloudProvider)
                            .enumeration(
                                CloudProvider::Enum,
                                [CloudProvider::Aws, CloudProvider::Gcp, CloudProvider::Azure],
                            )
                            .not_null(),
                    )
                    .col(string(Projects::Region))
                    .col(
                        ColumnDef::new(Projects::Environment)
                            .enumeration(
                                Environment::Enum,
                                [
                                    Environment::Development,
                                    Environment::Staging,
                                    Environment::Production,
                                ],
                            )
                            .not_null()
                            .default("development"),
                    )
                    .col(
                        ColumnDef::new(Projects::Status)
                            .enumeration(
                                ProjectStatus::Enum,
                                [
                                    ProjectStatus::Provisioning,
                                    ProjectStatus::Active,
                                    ProjectStatus::Suspended,
                                    ProjectStatus::Deleting,
                                    ProjectStatus::Archived,
                                ],
                            )
                            .not_null()
                            .default("provisioning"),
                    )
                    .col(double_null(Projects::BudgetLimit))
                    .col(json(Projects::Tags).default("[]"))
                    .col(boolean(Projects::Enabled).default(true))
                    .col(
                        timestamp_with_time_zone(Projects::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Projects::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_projects_user_id")
                    .table(Projects::Table)
                    .col(Projects::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_cloud_provider")
                    .table(Projects::Table)
                    .col(Projects::CloudProvider)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_status")
                    .table(Projects::Table)
                    .col(Projects::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_environment")
                    .table(Projects::Table)
                    .col(Projects::Environment)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_created_at")
                    .table(Projects::Table)
                    .col(Projects::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER projects_touch_updated_at
                    BEFORE UPDATE ON projects
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
            .execute_unprepared("DROP TRIGGER IF EXISTS projects_touch_updated_at ON projects")
            .await?;

        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Environment::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ProjectStatus::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(CloudProvider::Enum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    Name,
    UserId,
    Description,
    CloudProvider,
    Region,
    Environment,
    Status,
    BudgetLimit,
    Tags,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CloudProvider {
    #[sea_orm(iden = "cloud_provider")]
    Enum,
    #[sea_orm(iden = "aws")]
    Aws,
    #[sea_orm(iden = "gcp")]
    Gcp,
    #[sea_orm(iden = "azure")]
    Azure,
}

#[derive(DeriveIden)]
enum ProjectStatus {
    #[sea_orm(iden = "project_status")]
    Enum,
    #[sea_orm(iden = "provisioning")]
    Provisioning,
    #[sea_orm(iden = "active")]
    Active,
    #[sea_orm(iden = "suspended")]
    Suspended,
    #[sea_orm(iden = "deleting")]
    Deleting,
    #[sea_orm(iden = "archived")]
    Archived,
}

#[derive(DeriveIden)]
enum Environment {
    #[sea_orm(iden = "environment")]
    Enum,
    #[sea_orm(iden = "development")]
    Development,
    #[sea_orm(iden = "staging")]
    Staging,
    #[sea_orm(iden = "production")]
    Production,
}

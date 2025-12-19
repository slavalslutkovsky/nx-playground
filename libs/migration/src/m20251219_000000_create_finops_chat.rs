use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create finops_chat_sessions table
        manager
            .create_table(
                Table::create()
                    .table(FinopsChatSessions::Table)
                    .if_not_exists()
                    .col(pk_uuid(FinopsChatSessions::Id))
                    .col(ColumnDef::new(FinopsChatSessions::UserId).uuid().null())
                    .col(
                        ColumnDef::new(FinopsChatSessions::Title)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsChatSessions::Context)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(FinopsChatSessions::Status)
                            .string_len(50)
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsChatSessions::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsChatSessions::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_chat_sessions_user")
                            .from(FinopsChatSessions::Table, FinopsChatSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create finops_chat_messages table
        manager
            .create_table(
                Table::create()
                    .table(FinopsChatMessages::Table)
                    .if_not_exists()
                    .col(pk_uuid(FinopsChatMessages::Id))
                    .col(
                        ColumnDef::new(FinopsChatMessages::SessionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsChatMessages::Role)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(FinopsChatMessages::Content).text().null())
                    .col(
                        ColumnDef::new(FinopsChatMessages::ToolCalls)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsChatMessages::TokenCount)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsChatMessages::LatencyMs)
                            .integer()
                            .null(),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsChatMessages::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_chat_messages_session")
                            .from(FinopsChatMessages::Table, FinopsChatMessages::SessionId)
                            .to(FinopsChatSessions::Table, FinopsChatSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create finops_cloud_accounts table
        manager
            .create_table(
                Table::create()
                    .table(FinopsCloudAccounts::Table)
                    .if_not_exists()
                    .col(pk_uuid(FinopsCloudAccounts::Id))
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::Provider)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::AccountId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::Name)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::CredentialsEncrypted)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::Regions)
                            .array(ColumnType::Text)
                            .null(),
                    )
                    .col(timestamp_with_time_zone_null(
                        FinopsCloudAccounts::LastSyncAt,
                    ))
                    .col(
                        ColumnDef::new(FinopsCloudAccounts::Status)
                            .string_len(50)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsCloudAccounts::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_cloud_accounts_user")
                            .from(FinopsCloudAccounts::Table, FinopsCloudAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint for cloud accounts
        manager
            .create_index(
                Index::create()
                    .name("idx_finops_cloud_accounts_unique")
                    .table(FinopsCloudAccounts::Table)
                    .col(FinopsCloudAccounts::UserId)
                    .col(FinopsCloudAccounts::Provider)
                    .col(FinopsCloudAccounts::AccountId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create finops_resources table
        manager
            .create_table(
                Table::create()
                    .table(FinopsResources::Table)
                    .if_not_exists()
                    .col(pk_uuid(FinopsResources::Id))
                    .col(
                        ColumnDef::new(FinopsResources::AccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::ResourceId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::ResourceType)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::Region)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::Name)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::Specs)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::MonthlyCostCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsResources::Utilization)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(FinopsResources::Tags).json_binary().null())
                    .col(
                        timestamp_with_time_zone(FinopsResources::LastSeenAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsResources::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_resources_account")
                            .from(FinopsResources::Table, FinopsResources::AccountId)
                            .to(FinopsCloudAccounts::Table, FinopsCloudAccounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint for resources
        manager
            .create_index(
                Index::create()
                    .name("idx_finops_resources_unique")
                    .table(FinopsResources::Table)
                    .col(FinopsResources::AccountId)
                    .col(FinopsResources::ResourceId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create finops_recommendations table
        manager
            .create_table(
                Table::create()
                    .table(FinopsRecommendations::Table)
                    .if_not_exists()
                    .col(pk_uuid(FinopsRecommendations::Id))
                    .col(
                        ColumnDef::new(FinopsRecommendations::SessionId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::ResourceId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::RecommendationType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::Title)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::Description)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::CurrentCostCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::ProjectedCostCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::SavingsCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::Confidence)
                            .float()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::Details)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(FinopsRecommendations::Status)
                            .string_len(50)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        timestamp_with_time_zone(FinopsRecommendations::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_recommendations_session")
                            .from(
                                FinopsRecommendations::Table,
                                FinopsRecommendations::SessionId,
                            )
                            .to(FinopsChatSessions::Table, FinopsChatSessions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_finops_recommendations_resource")
                            .from(
                                FinopsRecommendations::Table,
                                FinopsRecommendations::ResourceId,
                            )
                            .to(FinopsResources::Table, FinopsResources::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_finops_chat_sessions_user")
                    .table(FinopsChatSessions::Table)
                    .col(FinopsChatSessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_chat_messages_session")
                    .table(FinopsChatMessages::Table)
                    .col(FinopsChatMessages::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_resources_account")
                    .table(FinopsResources::Table)
                    .col(FinopsResources::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_resources_type")
                    .table(FinopsResources::Table)
                    .col(FinopsResources::ResourceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_recommendations_session")
                    .table(FinopsRecommendations::Table)
                    .col(FinopsRecommendations::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_recommendations_resource")
                    .table(FinopsRecommendations::Table)
                    .col(FinopsRecommendations::ResourceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_finops_recommendations_type")
                    .table(FinopsRecommendations::Table)
                    .col(FinopsRecommendations::RecommendationType)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger for chat sessions
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER finops_chat_sessions_touch_updated_at
                    BEFORE UPDATE ON finops_chat_sessions
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
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS finops_chat_sessions_touch_updated_at ON finops_chat_sessions",
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(FinopsRecommendations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(FinopsResources::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(FinopsCloudAccounts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(FinopsChatMessages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(FinopsChatSessions::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Reference to users table for foreign keys.
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum FinopsChatSessions {
    Table,
    Id,
    UserId,
    Title,
    Context,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FinopsChatMessages {
    Table,
    Id,
    SessionId,
    Role,
    Content,
    ToolCalls,
    TokenCount,
    LatencyMs,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FinopsCloudAccounts {
    Table,
    Id,
    UserId,
    Provider,
    AccountId,
    Name,
    CredentialsEncrypted,
    Regions,
    LastSyncAt,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FinopsResources {
    Table,
    Id,
    AccountId,
    ResourceId,
    ResourceType,
    Region,
    Name,
    Specs,
    MonthlyCostCents,
    Utilization,
    Tags,
    LastSeenAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FinopsRecommendations {
    Table,
    Id,
    SessionId,
    ResourceId,
    RecommendationType,
    Title,
    Description,
    CurrentCostCents,
    ProjectedCostCents,
    SavingsCents,
    Confidence,
    Details,
    Status,
    CreatedAt,
}

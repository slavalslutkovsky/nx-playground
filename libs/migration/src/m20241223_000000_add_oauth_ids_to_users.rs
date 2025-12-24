use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add OAuth provider ID columns to users table
        // These allow direct lookup of users by OAuth provider ID

        // Add google_id column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::GoogleId).string().null())
                    .to_owned(),
            )
            .await?;

        // Add github_id column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::GithubId).string().null())
                    .to_owned(),
            )
            .await?;

        // Add workos_id column for WorkOS AuthKit
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::WorkosId).string().null())
                    .to_owned(),
            )
            .await?;

        // Create indexes for OAuth ID lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_users_google_id")
                    .table(Users::Table)
                    .col(Users::GoogleId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_github_id")
                    .table(Users::Table)
                    .col(Users::GithubId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_workos_id")
                    .table(Users::Table)
                    .col(Users::WorkosId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_workos_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_github_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_google_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::WorkosId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::GithubId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::GoogleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    GoogleId,
    GithubId,
    WorkosId,
}

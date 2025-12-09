use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add new authentication-related columns to users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    // Avatar URL (optional, from OAuth or user upload)
                    .add_column(ColumnDef::new(Users::AvatarUrl).text().null())
                    // Google OAuth ID (unique, nullable)
                    .add_column(
                        ColumnDef::new(Users::GoogleId)
                            .text()
                            .null()
                            .unique_key(),
                    )
                    // GitHub OAuth ID (unique, nullable)
                    .add_column(
                        ColumnDef::new(Users::GithubId)
                            .text()
                            .null()
                            .unique_key(),
                    )
                    // Last login timestamp
                    .add_column(timestamp_with_time_zone_null(Users::LastLoginAt))
                    // Account active status
                    .add_column(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // Account locked status
                    .add_column(
                        ColumnDef::new(Users::IsLocked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Failed login attempt counter
                    .add_column(
                        ColumnDef::new(Users::FailedLoginAttempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // Locked until timestamp (null if not locked)
                    .add_column(timestamp_with_time_zone_null(Users::LockedUntil))
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

        // Create index for locked_until to efficiently query locked accounts
        manager
            .create_index(
                Index::create()
                    .name("idx_users_locked_until")
                    .table(Users::Table)
                    .col(Users::LockedUntil)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_locked_until")
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
                    .drop_column(Users::LockedUntil)
                    .drop_column(Users::FailedLoginAttempts)
                    .drop_column(Users::IsLocked)
                    .drop_column(Users::IsActive)
                    .drop_column(Users::LastLoginAt)
                    .drop_column(Users::GithubId)
                    .drop_column(Users::GoogleId)
                    .drop_column(Users::AvatarUrl)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    AvatarUrl,
    GoogleId,
    GithubId,
    LastLoginAt,
    IsActive,
    IsLocked,
    FailedLoginAttempts,
    LockedUntil,
}

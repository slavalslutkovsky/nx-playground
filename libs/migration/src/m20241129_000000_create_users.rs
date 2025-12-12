use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create a users table with all auth-related fields
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id))
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(string(Users::Name))
                    .col(string(Users::PasswordHash))
                    .col(
                        ColumnDef::new(Users::Roles)
                            .array(ColumnType::Text)
                            .not_null()
                            .default(Expr::cust("ARRAY['user']::TEXT[]")),
                    )
                    .col(boolean(Users::EmailVerified).default(false))
                    .col(
                        timestamp_with_time_zone(Users::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Users::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    // Auth-related fields
                    .col(text_null(Users::AvatarUrl))
                    .col(timestamp_with_time_zone_null(Users::LastLoginAt))
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::IsLocked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::FailedLoginAttempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(timestamp_with_time_zone_null(Users::LockedUntil))
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_created_at")
                    .table(Users::Table)
                    .col(Users::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_locked_until")
                    .table(Users::Table)
                    .col(Users::LockedUntil)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER users_touch_updated_at
                    BEFORE UPDATE ON users
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
            .execute_unprepared("DROP TRIGGER IF EXISTS users_touch_updated_at ON users")
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Name,
    PasswordHash,
    Roles,
    EmailVerified,
    CreatedAt,
    UpdatedAt,
    AvatarUrl,
    LastLoginAt,
    IsActive,
    IsLocked,
    FailedLoginAttempts,
    LockedUntil,
}

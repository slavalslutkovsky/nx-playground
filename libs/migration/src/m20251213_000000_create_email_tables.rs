use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create a verification_tokens table
        manager
            .create_table(
                Table::create()
                    .table(VerificationTokens::Table)
                    .if_not_exists()
                    .col(pk_uuid(VerificationTokens::Id))
                    .col(
                        ColumnDef::new(VerificationTokens::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerificationTokens::Token)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(VerificationTokens::TokenType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerificationTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(timestamp_with_time_zone_null(VerificationTokens::UsedAt))
                    .col(
                        timestamp_with_time_zone(VerificationTokens::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_verification_tokens_user")
                            .from(VerificationTokens::Table, VerificationTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the email_logs table
        manager
            .create_table(
                Table::create()
                    .table(EmailLogs::Table)
                    .if_not_exists()
                    .col(pk_uuid(EmailLogs::Id))
                    .col(ColumnDef::new(EmailLogs::UserId).uuid().null())
                    .col(
                        ColumnDef::new(EmailLogs::EmailType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailLogs::ToEmail)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailLogs::Subject)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailLogs::Status)
                            .string_len(32)
                            .not_null()
                            .default("queued"),
                    )
                    .col(text_null(EmailLogs::ProviderMessageId))
                    .col(text_null(EmailLogs::ErrorMessage))
                    .col(
                        ColumnDef::new(EmailLogs::RetryCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(timestamp_with_time_zone_null(EmailLogs::SentAt))
                    .col(timestamp_with_time_zone_null(EmailLogs::DeliveredAt))
                    .col(timestamp_with_time_zone_null(EmailLogs::OpenedAt))
                    .col(
                        timestamp_with_time_zone(EmailLogs::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_logs_user")
                            .from(EmailLogs::Table, EmailLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the email_preferences table
        manager
            .create_table(
                Table::create()
                    .table(EmailPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EmailPreferences::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EmailPreferences::WelcomeEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(EmailPreferences::TaskNotificationsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(EmailPreferences::WeeklyDigestEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(EmailPreferences::MarketingEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        timestamp_with_time_zone(EmailPreferences::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_preferences_user")
                            .from(EmailPreferences::Table, EmailPreferences::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the email_suppressions table
        manager
            .create_table(
                Table::create()
                    .table(EmailSuppressions::Table)
                    .if_not_exists()
                    .col(pk_uuid(EmailSuppressions::Id))
                    .col(
                        ColumnDef::new(EmailSuppressions::Email)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EmailSuppressions::Reason)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(EmailSuppressions::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_user_id")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_token")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::Token)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_expires_at")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_logs_user_id")
                    .table(EmailLogs::Table)
                    .col(EmailLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_logs_to_email")
                    .table(EmailLogs::Table)
                    .col(EmailLogs::ToEmail)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_logs_status")
                    .table(EmailLogs::Table)
                    .col(EmailLogs::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_logs_created_at")
                    .table(EmailLogs::Table)
                    .col(EmailLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_suppressions_email")
                    .table(EmailSuppressions::Table)
                    .col(EmailSuppressions::Email)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger for email_preferences
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER email_preferences_touch_updated_at
                    BEFORE UPDATE ON email_preferences
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
                "DROP TRIGGER IF EXISTS email_preferences_touch_updated_at ON email_preferences",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(EmailSuppressions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EmailPreferences::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EmailLogs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(VerificationTokens::Table).to_owned())
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
enum VerificationTokens {
    Table,
    Id,
    UserId,
    Token,
    TokenType,
    ExpiresAt,
    UsedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum EmailLogs {
    Table,
    Id,
    UserId,
    EmailType,
    ToEmail,
    Subject,
    Status,
    ProviderMessageId,
    ErrorMessage,
    RetryCount,
    SentAt,
    DeliveredAt,
    OpenedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum EmailPreferences {
    Table,
    UserId,
    WelcomeEnabled,
    TaskNotificationsEnabled,
    WeeklyDigestEnabled,
    MarketingEnabled,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum EmailSuppressions {
    Table,
    Id,
    Email,
    Reason,
    CreatedAt,
}

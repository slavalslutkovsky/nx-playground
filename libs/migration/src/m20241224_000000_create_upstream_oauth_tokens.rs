use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table_name = Alias::new("upstream_oauth_tokens");

        // Create upstream_oauth_tokens table
        // Stores OAuth tokens from upstream providers (e.g., Google via WorkOS)
        // These tokens can be used to access external APIs like GCP
        manager
            .create_table(
                Table::create()
                    .table(table_name.clone())
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    // The upstream provider (google, github, etc.)
                    .col(ColumnDef::new(Alias::new("provider")).string().not_null())
                    // The auth provider that returned these tokens (workos, direct, etc.)
                    .col(
                        ColumnDef::new(Alias::new("auth_source"))
                            .string()
                            .not_null()
                            .default("workos"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("access_token"))
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("refresh_token")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("token_expires_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("scopes"))
                            .array(ColumnType::Text)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_upstream_oauth_tokens_user_id")
                            .from(table_name.clone(), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on user_id + provider (one token per provider per user)
        manager
            .create_index(
                Index::create()
                    .name("idx_upstream_oauth_tokens_user_provider")
                    .table(table_name.clone())
                    .col(Alias::new("user_id"))
                    .col(Alias::new("provider"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for fast lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_upstream_oauth_tokens_user_id")
                    .table(table_name.clone())
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("upstream_oauth_tokens"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

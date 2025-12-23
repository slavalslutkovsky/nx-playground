use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table_name = Alias::new("oauth_accounts");

        // Create oauth_accounts table
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
                    .col(ColumnDef::new(Alias::new("provider")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("provider_user_id"))
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("provider_username"))
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("email")).string().null())
                    .col(ColumnDef::new(Alias::new("display_name")).string().null())
                    .col(ColumnDef::new(Alias::new("avatar_url")).text().null())
                    .col(ColumnDef::new(Alias::new("access_token")).text().null())
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
                    .col(ColumnDef::new(Alias::new("raw_user_data")).json().null())
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
                            .name("fk_oauth_accounts_user_id")
                            .from(table_name.clone(), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on provider + provider_user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_accounts_provider_user")
                    .table(table_name.clone())
                    .col(Alias::new("provider"))
                    .col(Alias::new("provider_user_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for fast lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_accounts_user_id")
                    .table(table_name.clone())
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        // Create composite index on user_id + provider for unlinking
        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_accounts_user_provider")
                    .table(table_name.clone())
                    .col(Alias::new("user_id"))
                    .col(Alias::new("provider"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop oauth_accounts table (cascade will handle the indexes)
        manager
            .drop_table(Table::drop().table(Alias::new("oauth_accounts")).to_owned())
            .await?;

        Ok(())
    }
}

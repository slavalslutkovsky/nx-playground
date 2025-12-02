use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enable pgcrypto extension for UUID generation
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS pgcrypto")
            .await?;

        // Create util schema for utility functions
        manager
            .get_connection()
            .execute_unprepared("CREATE SCHEMA IF NOT EXISTS util")
            .await?;

        // Create touch_updated_at trigger function
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION util.touch_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql
                "#,
            )
            .await?;

        // Create notify_table_events trigger function for pub/sub
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION util.notify_table_events()
                RETURNS TRIGGER AS $$
                DECLARE
                    payload JSONB;
                BEGIN
                    payload := jsonb_build_object(
                        'table', TG_TABLE_NAME,
                        'action', TG_OP,
                        'data', CASE
                            WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW)
                            ELSE NULL
                        END,
                        'prev_data', CASE
                            WHEN TG_OP IN ('UPDATE', 'DELETE') THEN to_jsonb(OLD)
                            ELSE NULL
                        END
                    );

                    PERFORM pg_notify('table_events', payload::text);

                    IF TG_OP = 'DELETE' THEN
                        RETURN OLD;
                    END IF;

                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS util.notify_table_events()")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS util.touch_updated_at()")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP SCHEMA IF EXISTS util CASCADE")
            .await?;

        // Note: We don't drop pgcrypto as other databases might depend on it
        // manager
        //     .get_connection()
        //     .execute_unprepared("DROP EXTENSION IF EXISTS pgcrypto")
        //     .await?;

        Ok(())
    }
}

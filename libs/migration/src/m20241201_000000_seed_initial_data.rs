use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert sample users
        manager
            .get_connection()
            .execute_unprepared(r#"
            INSERT INTO users (id, email, name, password_hash, roles, email_verified, created_at, updated_at)
            VALUES
                (
                    '01930b3c-7c5f-7000-8000-000000000001',
                    'admin@example.com',
                    'Admin User',
                    '$argon2id$v=19$m=19456,t=2,p=1$VE0rHYzGbYjDhGgvhdzFPw$CJpleaNYKGFpc44EFOyWTE+fG2Z0A+6Ka2SlQQzroYA',
                    ARRAY['admin']::TEXT[],
                    true,
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-7001-8000-000000000002',
                    'user@example.com',
                    'Regular User',
                    '$argon2id$v=19$m=19456,t=2,p=1$VE0rHYzGbYjDhGgvhdzFPw$CJpleaNYKGFpc44EFOyWTE+fG2Z0A+6Ka2SlQQzroYA',
                    ARRAY['user']::TEXT[],
                    true,
                    NOW(),
                    NOW()
                )
            ON CONFLICT (id) DO NOTHING
            "#)
            .await?;

        // Insert sample projects
        manager
            .get_connection()
            .execute_unprepared(r#"
            INSERT INTO projects (
                id, name, user_id, description, cloud_provider, region,
                environment, status, budget_limit, tags, enabled, created_at, updated_at
            )
            VALUES
                (
                    '01930b3c-7c5f-7002-8000-000000000003',
                    'playground-monorepo',
                    '01930b3c-7c5f-7001-8000-000000000002',
                    'Main development playground for experimenting with Rust and Kubernetes',
                    'aws',
                    'us-east-1',
                    'development',
                    'active',
                    100.0,
                    '[]'::JSONB,
                    true,
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-7003-8000-000000000004',
                    'zerg-api-production',
                    '01930b3c-7c5f-7001-8000-000000000002',
                    'Production deployment of Zerg API services',
                    'aws',
                    'us-west-2',
                    'production',
                    'active',
                    500.0,
                    '[{"key": "team", "value": "platform"}, {"key": "criticality", "value": "high"}]'::JSONB,
                    true,
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-7004-8000-000000000005',
                    'ml-training-cluster',
                    '01930b3c-7c5f-7000-8000-000000000001',
                    'Machine learning model training infrastructure',
                    'gcp',
                    'us-central1',
                    'development',
                    'provisioning',
                    1000.0,
                    '[{"key": "type", "value": "ml"}, {"key": "gpu", "value": "true"}]'::JSONB,
                    true,
                    NOW(),
                    NOW()
                )
            ON CONFLICT (id) DO NOTHING
            "#)
            .await?;

        // Insert sample cloud resources
        manager
            .get_connection()
            .execute_unprepared(r#"
            INSERT INTO cloud_resources (
                id, project_id, name, resource_type, status, region,
                configuration, cost_per_hour, monthly_cost_estimate, tags,
                enabled, created_at, updated_at, deleted_at
            )
            VALUES
                (
                    '01930b3c-7c5f-7005-8000-000000000006',
                    '01930b3c-7c5f-7002-8000-000000000003',
                    'dev-postgres-primary',
                    'database',
                    'active',
                    'us-east-1',
                    '{"instance_type": "db.t3.medium", "engine": "postgres", "version": "15.3"}'::JSONB,
                    0.068,
                    48.96,
                    '[{"key": "backup", "value": "daily"}]'::JSONB,
                    true,
                    NOW(),
                    NOW(),
                    NULL
                ),
                (
                    '01930b3c-7c5f-7006-8000-000000000007',
                    '01930b3c-7c5f-7002-8000-000000000003',
                    'dev-redis-cache',
                    'database',
                    'active',
                    'us-east-1',
                    '{"instance_type": "cache.t3.micro", "engine": "redis", "version": "7.0"}'::JSONB,
                    0.017,
                    12.24,
                    '[]'::JSONB,
                    true,
                    NOW(),
                    NOW(),
                    NULL
                ),
                (
                    '01930b3c-7c5f-7007-8000-000000000008',
                    '01930b3c-7c5f-7003-8000-000000000004',
                    'prod-api-loadbalancer',
                    'network',
                    'active',
                    'us-west-2',
                    '{"type": "application", "scheme": "internet-facing", "ssl": true}'::JSONB,
                    0.025,
                    18.0,
                    '[{"key": "public", "value": "true"}]'::JSONB,
                    true,
                    NOW(),
                    NOW(),
                    NULL
                ),
                (
                    '01930b3c-7c5f-7008-8000-000000000009',
                    '01930b3c-7c5f-7004-8000-000000000005',
                    'ml-gpu-cluster',
                    'compute',
                    'creating',
                    'us-central1',
                    '{"instance_type": "n1-standard-16", "gpu": "nvidia-tesla-v100", "gpu_count": 4}'::JSONB,
                    12.5,
                    9000.0,
                    '[{"key": "gpu", "value": "v100"}, {"key": "count", "value": "4"}]'::JSONB,
                    true,
                    NOW(),
                    NOW(),
                    NULL
                )
            ON CONFLICT (id) DO NOTHING
            "#)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete in reverse order of foreign key dependencies
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM cloud_resources WHERE id LIKE '01930b3c-7c5f-7%'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DELETE FROM projects WHERE id LIKE '01930b3c-7c5f-7%'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DELETE FROM users WHERE id LIKE '01930b3c-7c5f-7%'")
            .await?;

        Ok(())
    }
}

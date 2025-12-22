use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert sample tasks
        manager
            .get_connection()
            .execute_unprepared(
                r#"
            INSERT INTO tasks (
                id, title, description, completed, project_id, priority, status,
                due_date, created_at, updated_at
            )
            VALUES
                (
                    '01930b3c-7c5f-7009-8000-000000000010',
                    'Setup CI/CD pipeline',
                    'Configure GitHub Actions for automated testing and deployment',
                    false,
                    '01930b3c-7c5f-7002-8000-000000000003',
                    'high'::task_priority,
                    'in_progress'::task_status,
                    NOW() + INTERVAL '7 days',
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-700a-8000-000000000011',
                    'Implement OAuth authentication',
                    'Add Google and GitHub OAuth support with PKCE',
                    true,
                    '01930b3c-7c5f-7003-8000-000000000004',
                    'high'::task_priority,
                    'done'::task_status,
                    NOW() - INTERVAL '2 days',
                    NOW() - INTERVAL '5 days',
                    NOW()
                ),
                (
                    '01930b3c-7c5f-700b-8000-000000000012',
                    'Database migration cleanup',
                    'Consolidate and optimize database migrations',
                    true,
                    '01930b3c-7c5f-7002-8000-000000000003',
                    'medium'::task_priority,
                    'done'::task_status,
                    NOW(),
                    NOW() - INTERVAL '1 day',
                    NOW()
                ),
                (
                    '01930b3c-7c5f-700c-8000-000000000013',
                    'Setup monitoring and alerts',
                    'Configure Prometheus and Grafana for production monitoring',
                    false,
                    '01930b3c-7c5f-7003-8000-000000000004',
                    'high'::task_priority,
                    'todo'::task_status,
                    NOW() + INTERVAL '14 days',
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-700d-8000-000000000014',
                    'Optimize API performance',
                    'Profile and optimize slow API endpoints, add caching',
                    false,
                    '01930b3c-7c5f-7003-8000-000000000004',
                    'medium'::task_priority,
                    'todo'::task_status,
                    NOW() + INTERVAL '21 days',
                    NOW(),
                    NOW()
                ),
                (
                    '01930b3c-7c5f-700e-8000-000000000015',
                    'ML model training infrastructure',
                    'Setup distributed training pipeline with GPU cluster',
                    false,
                    '01930b3c-7c5f-7004-8000-000000000005',
                    'urgent'::task_priority,
                    'in_progress'::task_status,
                    NOW() + INTERVAL '10 days',
                    NOW() - INTERVAL '3 days',
                    NOW()
                )
            ON CONFLICT (id) DO NOTHING
            "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM tasks WHERE id LIKE '01930b3c-7c5f-7%'")
            .await?;

        Ok(())
    }
}

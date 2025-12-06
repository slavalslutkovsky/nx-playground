use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create task_priority enum
        manager
            .create_type(
                Type::create()
                    .as_enum(TaskPriority::Enum)
                    .values([
                        TaskPriority::Low,
                        TaskPriority::Medium,
                        TaskPriority::High,
                        TaskPriority::Urgent,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create task_status enum
        manager
            .create_type(
                Type::create()
                    .as_enum(TaskStatus::Enum)
                    .values([
                        TaskStatus::Todo,
                        TaskStatus::InProgress,
                        TaskStatus::Done,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create tasks table
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(pk_uuid(Tasks::Id))
                    .col(string(Tasks::Title))
                    .col(string(Tasks::Description).default(""))
                    .col(boolean(Tasks::Completed).default(false))
                    .col(uuid_null(Tasks::ProjectId))
                    .col(
                        ColumnDef::new(Tasks::Priority)
                            .enumeration(
                                TaskPriority::Enum,
                                [
                                    TaskPriority::Low,
                                    TaskPriority::Medium,
                                    TaskPriority::High,
                                    TaskPriority::Urgent,
                                ],
                            )
                            .not_null()
                            .default("medium"),
                    )
                    .col(
                        ColumnDef::new(Tasks::Status)
                            .enumeration(
                                TaskStatus::Enum,
                                [
                                    TaskStatus::Todo,
                                    TaskStatus::InProgress,
                                    TaskStatus::Done,
                                ],
                            )
                            .not_null()
                            .default("todo"),
                    )
                    .col(timestamp_with_time_zone_null(Tasks::DueDate))
                    .col(
                        timestamp_with_time_zone(Tasks::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Tasks::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_project_id")
                            .from(Tasks::Table, Tasks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_project_id")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_status")
                    .table(Tasks::Table)
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_priority")
                    .table(Tasks::Table)
                    .col(Tasks::Priority)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_due_date")
                    .table(Tasks::Table)
                    .col(Tasks::DueDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_created_at")
                    .table(Tasks::Table)
                    .col(Tasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Add updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER tasks_touch_updated_at
                    BEFORE UPDATE ON tasks
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
            .execute_unprepared("DROP TRIGGER IF EXISTS tasks_touch_updated_at ON tasks")
            .await?;

        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TaskStatus::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TaskPriority::Enum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Title,
    Description,
    Completed,
    ProjectId,
    Priority,
    Status,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum TaskPriority {
    #[sea_orm(iden = "task_priority")]
    Enum,
    #[sea_orm(iden = "low")]
    Low,
    #[sea_orm(iden = "medium")]
    Medium,
    #[sea_orm(iden = "high")]
    High,
    #[sea_orm(iden = "urgent")]
    Urgent,
}

#[derive(DeriveIden)]
enum TaskStatus {
    #[sea_orm(iden = "task_status")]
    Enum,
    #[sea_orm(iden = "todo")]
    Todo,
    #[sea_orm(iden = "in_progress")]
    InProgress,
    #[sea_orm(iden = "done")]
    Done,
}

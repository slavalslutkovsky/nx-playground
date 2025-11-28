use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{ProjectError, ProjectResult};
use crate::models::{CreateProject, Project, ProjectFilter, Tag, UpdateProject};
use crate::repository::ProjectRepository;

/// PostgreSQL row representation
#[derive(Debug, FromRow)]
struct ProjectRow {
    id: Uuid,
    name: String,
    user_id: Uuid,
    description: String,
    cloud_provider: String,
    region: String,
    environment: String,
    status: String,
    budget_limit: Option<f64>,
    tags: sqlx::types::Json<Vec<Tag>>,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ProjectRow> for Project {
    type Error = ProjectError;

    fn try_from(row: ProjectRow) -> Result<Self, Self::Error> {
        Ok(Project {
            id: row.id,
            name: row.name,
            user_id: row.user_id,
            description: row.description,
            cloud_provider: row
                .cloud_provider
                .parse()
                .map_err(|e| ProjectError::Internal(format!("Invalid cloud_provider: {}", e)))?,
            region: row.region,
            environment: row
                .environment
                .parse()
                .map_err(|e| ProjectError::Internal(format!("Invalid environment: {}", e)))?,
            status: row
                .status
                .parse()
                .map_err(|e| ProjectError::Internal(format!("Invalid status: {}", e)))?,
            budget_limit: row.budget_limit,
            tags: row.tags.0,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// PostgreSQL implementation of ProjectRepository
#[derive(Clone)]
pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, input: CreateProject) -> ProjectResult<Project> {
        let id = Uuid::new_v4();
        let tags_json = serde_json::to_value(&input.tags)
            .map_err(|e| ProjectError::Internal(format!("Failed to serialize tags: {}", e)))?;

        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            INSERT INTO projects (id, name, user_id, description, cloud_provider, region, environment, status, budget_limit, tags, enabled)
            VALUES ($1, $2, $3, $4, $5::cloud_provider, $6, $7::environment, 'provisioning'::project_status, $8, $9, true)
            RETURNING id, name, user_id, description, cloud_provider::text, region, environment::text, status::text, budget_limit, tags, enabled, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.user_id)
        .bind(&input.description)
        .bind(input.cloud_provider.to_string())
        .bind(&input.region)
        .bind(input.environment.to_string())
        .bind(input.budget_limit)
        .bind(tags_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ProjectError::DuplicateName(input.name.clone())
            }
            _ => ProjectError::Internal(format!("Database error: {}", e)),
        })?;

        tracing::info!(project_id = %row.id, "Created project");
        row.try_into()
    }

    async fn get_by_id(&self, id: Uuid) -> ProjectResult<Option<Project>> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, user_id, description, cloud_provider::text, region, environment::text, status::text, budget_limit, tags, enabled, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn list(&self, filter: ProjectFilter) -> ProjectResult<Vec<Project>> {
        // Use static query with optional filters applied via COALESCE/CASE
        // This approach avoids dynamic SQL and lifetime issues
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, user_id, description, cloud_provider::text, region, environment::text, status::text, budget_limit, tags, enabled, created_at, updated_at
            FROM projects
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR cloud_provider::text = $2)
              AND ($3::text IS NULL OR environment::text = $3)
              AND ($4::text IS NULL OR status::text = $4)
              AND ($5::boolean IS NULL OR enabled = $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(filter.user_id)
        .bind(filter.cloud_provider.map(|p| p.to_string()))
        .bind(filter.environment.map(|e| e.to_string()))
        .bind(filter.status.map(|s| s.to_string()))
        .bind(filter.enabled)
        .bind(filter.limit as i64)
        .bind(filter.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, id: Uuid, input: UpdateProject) -> ProjectResult<Project> {
        // Fetch existing project
        let mut project = self
            .get_by_id(id)
            .await?
            .ok_or(ProjectError::NotFound(id))?;

        // Apply updates in memory
        project.apply_update(input.clone());

        // Serialize tags
        let tags_json = serde_json::to_value(&project.tags)
            .map_err(|e| ProjectError::Internal(format!("Failed to serialize tags: {}", e)))?;

        // Update all fields using COALESCE pattern
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            UPDATE projects
            SET name = $1,
                description = $2,
                region = $3,
                environment = $4::environment,
                status = $5::project_status,
                budget_limit = $6,
                tags = $7,
                enabled = $8
            WHERE id = $9
            RETURNING id, name, user_id, description, cloud_provider::text, region, environment::text, status::text, budget_limit, tags, enabled, created_at, updated_at
            "#,
        )
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.region)
        .bind(project.environment.to_string())
        .bind(project.status.to_string())
        .bind(project.budget_limit)
        .bind(tags_json)
        .bind(project.enabled)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ProjectError::DuplicateName(input.name.unwrap_or_default())
                }
                _ => ProjectError::Internal(format!("Database error: {}", e)),
            })?;

        tracing::info!(project_id = %id, "Updated project");
        row.try_into()
    }

    async fn delete(&self, id: Uuid) -> ProjectResult<bool> {
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        if result.rows_affected() > 0 {
            tracing::info!(project_id = %id, "Deleted project");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists_by_name(&self, user_id: Uuid, name: &str) -> ProjectResult<bool> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM projects WHERE user_id = $1 AND LOWER(name) = LOWER($2)",
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ProjectError::Internal(format!("Database error: {}", e)))?;

        Ok(result.0 > 0)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require a test database
    // Consider using testcontainers for integration testing
}

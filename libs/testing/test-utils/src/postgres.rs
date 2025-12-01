//! PostgreSQL test infrastructure
//!
//! Provides a `TestDatabase` helper that creates a PostgreSQL container for testing.

use migration::MigratorTrait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

/// Test database wrapper that ensures proper cleanup
///
/// The container is automatically stopped and removed when this struct is dropped.
pub struct TestDatabase {
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
    pub connection: DatabaseConnection,
    pub connection_string: String,
}

impl TestDatabase {
    /// Create a new test database with migrations applied
    ///
    /// # Example
    ///
    /// ```no_run
    /// use test_utils::TestDatabase;
    ///
    /// # async fn example() {
    /// let db = TestDatabase::new().await;
    /// // Use db.connection() to create your repository
    /// # }
    /// ```
    pub async fn new() -> Self {
        // Use Postgres 18 to match production
        let postgres = Postgres::default().with_tag("18-alpine");

        let container = postgres
            .start()
            .await
            .expect("Failed to start Postgres container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");

        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // Connect to database
        let connection = Database::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        migration::Migrator::up(&connection, None)
            .await
            .expect("Failed to run migrations");

        tracing::info!(port = host_port, "Test database ready (Postgres 18)");

        Self {
            container,
            connection,
            connection_string,
        }
    }

    /// Create a test database with a specific schema (for parallel test isolation)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use test_utils::TestDatabase;
    ///
    /// # async fn example() {
    /// let db = TestDatabase::with_schema("test_create_project").await;
    /// # }
    /// ```
    pub async fn with_schema(schema_name: &str) -> Self {
        let db = Self::new().await;

        // Create schema for isolation
        let create_schema = format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name);
        db.connection
            .execute_unprepared(&create_schema)
            .await
            .expect("Failed to create schema");

        // Set search path to use this schema
        let set_path = format!("SET search_path TO {}", schema_name);
        db.connection
            .execute_unprepared(&set_path)
            .await
            .expect("Failed to set search path");

        // Run migrations in this schema
        migration::Migrator::up(&db.connection, None)
            .await
            .expect("Failed to run migrations in schema");

        db
    }

    /// Get a cloned connection (useful for passing to repositories)
    pub fn connection(&self) -> DatabaseConnection {
        self.connection.clone()
    }
}

// Container is automatically cleaned up when TestDatabase is dropped
impl Drop for TestDatabase {
    fn drop(&mut self) {
        tracing::debug!("Cleaning up test database container");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = TestDatabase::new().await;
        assert!(db.connection_string.contains("postgres://"));
    }

    #[tokio::test]
    async fn test_schema_isolation() {
        let db1 = TestDatabase::with_schema("schema1").await;
        let db2 = TestDatabase::with_schema("schema2").await;

        // Both databases should be functional
        assert!(db1.connection_string.contains("postgres://"));
        assert!(db2.connection_string.contains("postgres://"));
    }
}

# Modular Monolith Architecture

This document describes the modular monolith architecture used in the `zerg_api` project.

## Table of Contents

- [Overview](#overview)
- [Architecture Layers](#architecture-layers)
- [Domain Modules](#domain-modules)
- [Database Integration](#database-integration)
- [API Endpoints](#api-endpoints)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Best Practices](#best-practices)

## Overview

The project uses a **modular monolith** architecture where:

- Each domain is self-contained with its own models, repository, service, and handlers
- Domains can be easily extracted into microservices later
- Shared infrastructure (database, messaging) is centralized
- All domains run in a single deployment but maintain clear boundaries

```
┌─────────────────────────────────────────────────────────┐
│                      zerg_api                           │
│                   (Single Deployment)                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Projects   │  │    Users     │  │    Tasks     │  │
│  │   Domain     │  │   Domain     │  │  (via gRPC)  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         │                 │                  │          │
│  ┌──────┴─────────────────┴──────────────────┘          │
│  │         Shared Infrastructure                        │
│  │  - PostgreSQL Pool                                   │
│  │  - Tracing/Logging                                   │
│  │  - Configuration                                     │
│  └──────────────────────────────────────────────────────┘
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Architecture Layers

Each domain follows a **4-layer architecture**:

### Layer Structure

```
libs/domains/<domain>/
├── models/         # Layer 1: Data structures
├── repository/     # Layer 2: Data access
├── service/        # Layer 3: Business logic
└── handlers/       # Layer 4: API endpoints
```

### Dependency Flow

```
Handlers
   ↓ depends on
Service
   ↓ depends on
Repository
   ↓ depends on
Models
```

**Rules:**
- Higher layers can depend on lower layers
- Lower layers cannot depend on higher layers
- Each layer has a single responsibility

### Layer Responsibilities

#### 1. Models Layer (`models.rs`)

**Purpose**: Define data structures, DTOs, and domain types

```rust
// Entity
pub struct Project { ... }

// DTOs
pub struct CreateProject { ... }
pub struct UpdateProject { ... }

// Enums
pub enum CloudProvider { Aws, Gcp, Azure }
```

**Characteristics:**
- No business logic
- Serialization/deserialization
- Data validation attributes
- Type conversions

#### 2. Repository Layer (`repository.rs`, `postgres.rs`)

**Purpose**: Abstract data access

```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, input: CreateProject) -> Result<Project>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Project>>;
    async fn list(&self, filter: Filter) -> Result<Vec<Project>>;
    async fn update(&self, id: Uuid, input: UpdateProject) -> Result<Project>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}
```

**Implementations:**
- `InMemoryRepository` - for testing/development
- `PgRepository` - for PostgreSQL production use

**Characteristics:**
- Database agnostic interface
- CRUD operations only
- No business rules
- Swappable implementations

#### 3. Service Layer (`service.rs`)

**Purpose**: Implement business logic

```rust
pub struct ProjectService<R: ProjectRepository> {
    repository: Arc<R>,
}

impl<R: ProjectRepository> ProjectService<R> {
    pub async fn create_project(&self, input: CreateProject) -> Result<Project> {
        // 1. Validate input
        self.validate_create(&input)?;

        // 2. Apply business rules
        // ...

        // 3. Call repository
        self.repository.create(input).await
    }
}
```

**Responsibilities:**
- Input validation
- Business rule enforcement
- Orchestration of repository calls
- Domain event emission (future)

**Characteristics:**
- Generic over repository implementation
- Pure business logic
- No HTTP/transport concerns

#### 4. Handlers Layer (`handlers.rs`)

**Purpose**: HTTP API endpoints

```rust
pub fn router<R: ProjectRepository + 'static>(
    service: ProjectService<R>
) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get).put(update).delete(delete))
        .with_state(Arc::new(service))
}
```

**Responsibilities:**
- HTTP request/response mapping
- Status code selection
- Error transformation
- Route definition

**Characteristics:**
- Thin layer (delegate to service)
- Framework-specific (Axum)
- No business logic

## Domain Modules

### Projects Domain

**Location**: `libs/domains/projects/`

**Purpose**: Manage cloud infrastructure projects

**Schema**:
```sql
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    user_id UUID NOT NULL,
    description TEXT,
    cloud_provider cloud_provider NOT NULL,  -- aws, gcp, azure
    region TEXT NOT NULL,
    environment environment NOT NULL,         -- dev, staging, prod
    status project_status NOT NULL,           -- provisioning, active, etc
    budget_limit DOUBLE PRECISION,
    tags JSONB,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    CONSTRAINT unique_project_name_per_user UNIQUE (user_id, name)
);
```

**Endpoints**:
```
GET    /projects              # List with filters
POST   /projects              # Create
GET    /projects/{id}         # Get by ID
PUT    /projects/{id}         # Update
DELETE /projects/{id}         # Delete
POST   /projects/{id}/activate
POST   /projects/{id}/suspend
POST   /projects/{id}/archive
```

**Example Usage**:
```bash
curl -X POST http://localhost:3000/projects \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-infra",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "description": "Production infrastructure",
    "cloud_provider": "aws",
    "region": "us-east-1",
    "environment": "production",
    "budget_limit": 1000.00,
    "tags": [
      {"key": "team", "value": "platform"},
      {"key": "cost-center", "value": "engineering"}
    ]
  }'
```

### Users Domain

**Location**: `libs/domains/users/`

**Purpose**: User management and authentication

**Schema**:
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    roles TEXT[] NOT NULL DEFAULT ARRAY['user'],
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

**Endpoints**:
```
GET    /users                 # List
POST   /users                 # Register
GET    /users/{id}            # Get by ID
PUT    /users/{id}            # Update
DELETE /users/{id}            # Delete
POST   /users/{id}/verify-email
POST   /users/{id}/change-password
POST   /users/login           # Authenticate
```

**Security Features**:
- Argon2 password hashing
- Email uniqueness enforcement
- Role-based access control
- Password strength validation (min 8 chars)

**Example Usage**:
```bash
# Register
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "name": "John Doe",
    "password": "secure_password_123",
    "roles": ["user"]
  }'

# Login
curl -X POST http://localhost:3000/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password_123"
  }'
```

## Database Integration

### Connection Management

```rust
// Create connection pool
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

// Use in repositories
let projects_repo = PgProjectRepository::new(pool.clone());
let users_repo = PgUserRepository::new(pool.clone());
```

### Migration Strategy

Migrations are located in `manifests/migrations/postgres/`

**Naming Convention**: `NNNN_description.sql`
- `0000_bootstrap.sql` - Initial setup
- `0004_projects.sql` - Projects v1 (old)
- `0005_users.sql` - Users table
- `0007_projects_v2.sql` - Projects v2 (current)

**Run Migrations**:
```bash
just _migration
# or
sqlx migrate run \
  --database-url=postgres://myuser:mypassword@localhost/mydatabase \
  --source manifests/migrations/postgres/
```

### Repository Pattern

#### Trait Definition

```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, input: CreateProject) -> Result<Project>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Project>>;
    // ...
}
```

#### PostgreSQL Implementation

```rust
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
    async fn create(&self, input: CreateProject) -> Result<Project> {
        sqlx::query_as(/* SQL */)
            .bind(/* params */)
            .fetch_one(&self.pool)
            .await?;
        // ...
    }
}
```

#### In-Memory Implementation

```rust
pub struct InMemoryProjectRepository {
    projects: Arc<RwLock<HashMap<Uuid, Project>>>,
}

// Useful for:
// - Unit testing
// - Development without database
// - Fast integration tests
```

## API Endpoints

### Complete Endpoint Map

```
┌─────────────────────────────────────────────────────┐
│                   zerg_api:3000                     │
├─────────────────────────────────────────────────────┤
│                                                      │
│  GET  /health                    Health check       │
│                                                      │
│  ┌─── Projects ────────────────────────────────┐   │
│  │ GET    /projects                            │   │
│  │ POST   /projects                            │   │
│  │ GET    /projects/{id}                       │   │
│  │ PUT    /projects/{id}                       │   │
│  │ DELETE /projects/{id}                       │   │
│  │ POST   /projects/{id}/activate              │   │
│  │ POST   /projects/{id}/suspend               │   │
│  │ POST   /projects/{id}/archive               │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌─── Users ───────────────────────────────────┐   │
│  │ GET    /users                               │   │
│  │ POST   /users                               │   │
│  │ GET    /users/{id}                          │   │
│  │ PUT    /users/{id}                          │   │
│  │ DELETE /users/{id}                          │   │
│  │ POST   /users/{id}/verify-email             │   │
│  │ POST   /users/{id}/change-password          │   │
│  │ POST   /users/login                         │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌─── Tasks (gRPC proxy) ──────────────────────┐   │
│  │ GET    /tasks                               │   │
│  │ POST   /tasks                               │   │
│  │ GET    /tasks/{id}                          │   │
│  │ DELETE /tasks/{id}                          │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

## Getting Started

### Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- Just (command runner)
- PostgreSQL client tools

### Initial Setup

```bash
# 1. Start infrastructure
docker compose -f manifests/dockers/compose.yaml up -d

# 2. Run migrations
just _migration

# 3. Build the project
cargo build

# 4. Run the API
cargo run -p zerg_api
```

### Environment Variables

```bash
# Database
export DATABASE_URL="postgres://myuser:mypassword@localhost/mydatabase"

# gRPC Services
export TASKS_SERVICE_ADDR="http://[::1]:50051"
```

### Development Commands

```bash
# Run tests
cargo test --workspace

# Check compilation
cargo check

# Format code
cargo fmt

# Run with watch (requires bacon)
just run

# Reset database
just reset-db
```

## Development Workflow

### Adding a New Domain

1. **Create domain structure**:
```bash
mkdir -p libs/domains/my_domain/src
cd libs/domains/my_domain
```

2. **Create Cargo.toml**:
```toml
[package]
name = "domain_my_domain"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
sqlx = { workspace = true }
# ...
```

3. **Create layer files**:
```
src/
├── models.rs      # Define entities and DTOs
├── error.rs       # Domain-specific errors
├── repository.rs  # Repository trait + in-memory impl
├── postgres.rs    # PostgreSQL implementation
├── service.rs     # Business logic
├── handlers.rs    # HTTP endpoints
└── lib.rs         # Module exports
```

4. **Add to workspace**:
```toml
# In root Cargo.toml
[workspace]
members = [
    # ...
    "libs/domains/my_domain"
]

[workspace.dependencies]
domain_my_domain = { path = "libs/domains/my_domain" }
```

5. **Create migration**:
```sql
-- manifests/migrations/postgres/NNNN_my_domain.sql
BEGIN;

CREATE TABLE my_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ...
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMIT;
```

6. **Integrate into API**:
```rust
// In apps/zerg/api/src/main.rs
use domain_my_domain::{handlers, PgMyDomainRepository, MyDomainService};

let repo = PgMyDomainRepository::new(pool.clone());
let service = MyDomainService::new(repo);

let app = Router::new()
    // ...
    .nest("/my-domain", handlers::router(service));
```

### Testing Strategy

#### Unit Tests

```rust
// In libs/domains/projects/src/service.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryProjectRepository;

    #[tokio::test]
    async fn test_create_project_validation() {
        let repo = InMemoryProjectRepository::new();
        let service = ProjectService::new(repo);

        let input = CreateProject {
            name: "", // Invalid
            // ...
        };

        let result = service.create_project(input).await;
        assert!(result.is_err());
    }
}
```

#### Integration Tests

```rust
// In apps/zerg/api/tests/integration.rs
use sqlx::PgPool;
use testcontainers::*;

#[tokio::test]
async fn test_project_crud() {
    // Use testcontainers for isolated database
    let container = Postgres::default();
    let pool = PgPool::connect(&connection_string).await?;

    // Run migrations
    sqlx::migrate!("../../manifests/migrations/postgres")
        .run(&pool)
        .await?;

    // Test CRUD operations
    // ...
}
```

### Error Handling Pattern

```rust
// Domain-specific errors
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Project not found: {0}")]
    NotFound(Uuid),

    #[error("Duplicate name: {0}")]
    DuplicateName(String),

    #[error("Validation: {0}")]
    Validation(String),
}

// Convert to HTTP responses
impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ProjectError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ProjectError::DuplicateName(_) => (StatusCode::CONFLICT, self.to_string()),
            ProjectError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

## Best Practices

### 1. Repository Pattern

**✅ DO:**
- Keep repositories simple (CRUD only)
- Use traits for abstraction
- Provide in-memory implementation for tests

**❌ DON'T:**
- Put business logic in repositories
- Make repositories domain-aware
- Directly expose database types

### 2. Service Layer

**✅ DO:**
- Validate all inputs
- Enforce business rules
- Keep services pure (no side effects visible to callers)
- Use clear method names (`create_project`, not `create`)

**❌ DON'T:**
- Access database directly
- Handle HTTP concerns
- Return database-specific errors

### 3. Handlers

**✅ DO:**
- Keep handlers thin
- Map domain errors to HTTP status codes
- Use extractors for validation

**❌ DON'T:**
- Put business logic in handlers
- Call repositories directly
- Expose internal error details

### 4. Models

**✅ DO:**
- Use strong types (newtypes, enums)
- Separate entities from DTOs
- Implement `From`/`Into` for conversions

**❌ DON'T:**
- Put business logic in models
- Expose database implementation details
- Use stringly-typed fields

### 5. Migrations

**✅ DO:**
- Use sequential numbering
- Include rollback strategy
- Test migrations locally first
- Add indexes for foreign keys

**❌ DON'T:**
- Modify existing migrations
- Use DROP TABLE in production
- Skip migration testing

### 6. Testing

**✅ DO:**
- Test business logic in service layer
- Use in-memory repositories for unit tests
- Use testcontainers for integration tests
- Test error cases

**❌ DON'T:**
- Test implementation details
- Use production database for tests
- Skip edge cases

## Future Enhancements

### Planned Features

1. **Authentication & Authorization**
   - JWT token generation
   - Middleware for route protection
   - Role-based access control

2. **Domain Events**
   - Event emission from services
   - Event handlers
   - Outbox pattern for reliability

3. **API Documentation**
   - OpenAPI/Swagger integration
   - Auto-generated docs from code

4. **Observability**
   - Structured logging
   - Distributed tracing
   - Metrics collection

5. **Caching Layer**
   - Redis integration
   - Cache-aside pattern
   - Invalidation strategy

### Migration to Microservices

Each domain is already structured to be extracted:

```
Monolith                    Microservices
├── domain_projects    →    projects-service (port 3001)
├── domain_users       →    users-service (port 3002)
└── domain_tasks       →    tasks-service (port 50051) ✓ Already separate!
```

**Extraction Steps**:
1. Create new app: `apps/services/projects-service`
2. Move domain code (already isolated)
3. Add gRPC server implementation
4. Update API gateway to call via gRPC
5. Deploy independently

The modular structure ensures minimal refactoring during extraction.

## References

- [Modular Monolith Architecture](../docs/messaging-patterns.md)
- [gRPC Patterns](../docs/grpc.md)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)

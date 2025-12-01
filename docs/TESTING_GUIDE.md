# Testing Guide

Complete guide to testing in the nx-playground monorepo.

## Table of Contents

1. [Testing Pyramid](#testing-pyramid)
2. [Test Types](#test-types)
3. [When to Use Each Type](#when-to-use-each-type)
4. [Running Tests](#running-tests)
5. [Writing Tests](#writing-tests)
6. [Best Practices](#best-practices)

---

## Testing Pyramid

```
           /\
          /  \     E2E Tests (few, slow, high confidence)
         / 10 \    apps/zerg/api/tests/e2e_test.rs
        /──────\
       /        \  Handler Tests (some, medium, domain APIs)
      /   30     \ libs/domains/*/tests/handler_test.rs
     /────────────\
    /              \ Integration Tests (many, medium, DB operations)
   /      100       \ libs/domains/*/tests/integration_test.rs
  /──────────────────\
 /                    \ Unit Tests (most, fast, business logic)
/         300          \ libs/domains/*/src/**/tests
────────────────────────

Total: ~440 tests across the pyramid
```

---

## Test Types

### 1. Unit Tests

**Location:** `libs/domains/*/src/service.rs` (and other source files)

**What they test:**
- Business logic in isolation
- Service methods with mocked dependencies
- Pure functions

**Example:**
```rust
// libs/domains/projects/src/service.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::MockProjectRepository;

    #[tokio::test]
    async fn test_can_create_project_when_under_limit() {
        let mut mock_repo = MockProjectRepository::new();

        // Mock: user has 2 projects
        mock_repo
            .expect_count_by_user()
            .returning(|_| Ok(2));

        let service = ProjectService::new(mock_repo);
        let can_create = service.can_user_create_project(user_id).await.unwrap();

        assert!(can_create);
    }
}
```

**Characteristics:**
- ⚡ **Speed:** 0.00s (instant)
- 🔧 **Dependencies:** Mocked (mockall)
- 🎯 **Scope:** Single function/method
- 📦 **Database:** No
- 🌐 **HTTP:** No

**Run with:**
```bash
cargo test -p domain_projects --lib
```

---

### 2. Integration Tests

**Location:** `libs/domains/*/tests/integration_test.rs`

**What they test:**
- Service + Repository + Database working together
- Database queries and constraints
- Transaction behavior
- Data persistence

**Example:**
```rust
// libs/domains/projects/tests/integration_test.rs

#[tokio::test]
async fn test_free_tier_cannot_create_4th_project() {
    let db = TestDatabase::new().await;  // Real Postgres!
    let repo = PgProjectRepository::new(db.connection());
    let service = ProjectService::new(repo);

    // Create 3 projects in real database
    for i in 0..3 {
        service.create_project(input).await.unwrap();
    }

    // Try to create 4th - should fail
    let result = service.create_project(input).await;
    assert!(matches!(result, Err(ProjectError::Validation(_))));
}
```

**Characteristics:**
- ⏱️ **Speed:** ~10s (for 12 tests)
- 🔧 **Dependencies:** Real database (testcontainers)
- 🎯 **Scope:** Service → Repository → Database
- 📦 **Database:** Yes (Postgres 18)
- 🌐 **HTTP:** No

**Run with:**
```bash
cargo test -p domain_projects --test integration_test
```

---

### 3. Handler Tests

**Location:** `libs/domains/*/tests/handler_test.rs`

**What they test:**
- HTTP handlers for a single domain
- Request/response serialization
- HTTP status codes
- Input validation at HTTP layer

**Example:**
```rust
// libs/domains/projects/tests/handler_test.rs

#[tokio::test]
async fn test_create_project_handler_returns_201() {
    let db = TestDatabase::new().await;
    let repo = PgProjectRepository::new(db.connection());
    let service = ProjectService::new(repo);
    let app = handlers::router(service);  // Only projects router!

    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("content-type", "application/json")
        .body(Body::from(json_string))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

**Characteristics:**
- ⏱️ **Speed:** ~2-5s (for 7 tests)
- 🔧 **Dependencies:** Real database
- 🎯 **Scope:** HTTP handlers for ONE domain
- 📦 **Database:** Yes
- 🌐 **HTTP:** Yes (domain-level routing only)

**Run with:**
```bash
cargo test -p domain_projects --test handler_test
```

---

### 4. E2E Tests

**Location:** `apps/zerg/api/tests/e2e_test.rs`

**What they test:**
- **Full application** with all domains
- Application-level routing (`/api/projects`, `/api/cloud-resources`)
- Authentication/authorization middleware
- Cross-domain interactions
- Exactly what users experience

**Example:**
```rust
// apps/zerg/api/tests/e2e_test.rs

#[tokio::test]
async fn e2e_free_tier_limit_enforced() {
    let app = start_test_app().await;  // Full app with all domains!
    let token = create_test_jwt(user_id);

    // Create 3 projects via full API
    for i in 0..3 {
        let response = app
            .post("/api/projects")  // Full route with /api prefix
            .header("Authorization", format!("Bearer {}", token))
            .json(&project_data)
            .send()
            .await;
        assert_eq!(response.status(), 201);
    }

    // 4th should fail with proper auth error
    let response = app
        .post("/api/projects")
        .header("Authorization", format!("Bearer {}", token))
        .json(&project_data)
        .send()
        .await;
    assert_eq!(response.status(), 400);
}
```

**Characteristics:**
- 🐌 **Speed:** ~15-30s (for 10 tests)
- 🔧 **Dependencies:** Real database + full app stack
- 🎯 **Scope:** Entire application
- 📦 **Database:** Yes
- 🌐 **HTTP:** Yes (full routing + middleware)

**Run with:**
```bash
cargo test -p zerg_api --test e2e_test
```

---

## When to Use Each Type

| Scenario | Test Type | Why? |
|----------|-----------|------|
| Testing business logic | **Unit** | Fast, isolated, easy to test edge cases |
| Checking if SQL query works | **Integration** | Catches database-specific bugs |
| Testing JSON serialization | **Handler** | Verifies HTTP layer works |
| Testing auth middleware | **E2E** | Only E2E tests full middleware stack |
| Testing 3-project limit logic | **Unit** | Business rule - mock returns different counts |
| Verifying 3-project limit works | **Integration** | Actual database count |
| Testing limit via API | **Handler** | HTTP 400 response |
| Testing limit with auth | **E2E** | Full user experience |

---

## Running Tests

### Run All Tests
```bash
# Everything
cargo test

# Specific domain
cargo test -p domain_projects

# Specific domain, all types
cargo test -p domain_projects --all-targets
```

### Run by Type
```bash
# Unit tests only
cargo test -p domain_projects --lib

# Integration tests only
cargo test -p domain_projects --test integration_test

# Handler tests only
cargo test -p domain_projects --test handler_test

# E2E tests only
cargo test -p zerg_api --test e2e_test
```

### Run Specific Test
```bash
cargo test -p domain_projects test_free_tier_cannot_create_4th_project
```

### Run with Output
```bash
cargo test -- --nocapture
```

### Run in Parallel
```bash
# Default: parallel (fast but can cause issues)
cargo test

# Sequential (slower but safer)
cargo test -- --test-threads=1

# Limited parallelism
cargo test -- --test-threads=4
```

---

## Writing Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::MockProjectRepository;

    #[tokio::test]
    async fn test_descriptive_name() {
        // Arrange
        let mut mock_repo = MockProjectRepository::new();
        mock_repo.expect_method().returning(|_| Ok(value));
        let service = Service::new(mock_repo);

        // Act
        let result = service.method().await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
```

### Integration Test Template

```rust
use test_utils::{TestDatabase, TestDataBuilder};

#[tokio::test]
async fn test_descriptive_name() {
    // Arrange
    let db = TestDatabase::new().await;
    let repo = Repository::new(db.connection());
    let builder = TestDataBuilder::from_test_name("unique_test_name");

    // Act
    let result = repo.method(builder.user_id()).await;

    // Assert
    assert!(result.is_ok());
}
```

### Handler Test Template

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_handler_descriptive_name() {
    // Arrange
    let db = TestDatabase::new().await;
    let service = Service::new(repo);
    let app = handlers::router(service);

    // Act
    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("content-type", "application/json")
        .body(Body::from(json_string))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

---

## Best Practices

### 1. Test Naming
```rust
// ✅ Good: Describes what and expected outcome
test_can_create_project_when_under_limit()
test_free_tier_cannot_create_4th_project()
test_create_project_handler_returns_201()

// ❌ Bad: Vague or unclear
test_project()
test_service()
test_handler()
```

### 2. Test Independence
```rust
// ✅ Good: Each test creates its own data
#[tokio::test]
async fn test_something() {
    let db = TestDatabase::new().await;  // Fresh database
    let builder = TestDataBuilder::from_test_name("unique_name");
}

// ❌ Bad: Tests share data (flaky!)
static SHARED_USER_ID: Uuid = ...;  // Don't do this!
```

### 3. Assertion Messages
```rust
// ✅ Good: Clear context
assert_eq!(
    projects.len(), 3,
    "User should have exactly 3 projects after limit"
);

// ❌ Bad: No context
assert_eq!(projects.len(), 3);
```

### 4. Test Data Builders
```rust
// ✅ Good: Deterministic, readable
let builder = TestDataBuilder::from_test_name("my_test");
let name = builder.name("project", "main");
let user_id = builder.user_id();

// ❌ Bad: Random UUIDs (non-deterministic)
let name = format!("project-{}", Uuid::new_v4());
let user_id = Uuid::new_v4();
```

### 5. Test Organization
```
libs/domains/projects/
├── src/
│   ├── service.rs       ← Unit tests here (#[cfg(test)] mod tests)
│   ├── models.rs        ← Unit tests here
│   └── lib.rs
└── tests/
    ├── integration_test.rs  ← Integration tests
    ├── handler_test.rs      ← Handler tests
    └── common/
        └── mod.rs           ← Shared test helpers (DEPRECATED - use test-utils)
```

### 6. Mocking Strategy
```rust
// ✅ Good: Mock external dependencies
let mut mock_repo = MockProjectRepository::new();
let mut mock_billing = MockBillingService::new();

// ❌ Bad: Don't mock what you're testing
let mut mock_service = MockProjectService::new();  // Testing the service!
```

---

## Test Coverage Goals

| Test Type | Coverage Goal | Current | Notes |
|-----------|---------------|---------|-------|
| **Unit** | 80%+ | ~75% | Focus on business logic |
| **Integration** | Critical paths | ✅ | All CRUD + constraints |
| **Handler** | All endpoints | ✅ | All HTTP status codes |
| **E2E** | User journeys | 🚧 | Planned, not implemented |

---

## Troubleshooting

### Tests Hang
```bash
# Check for orphaned containers
docker ps -a | grep postgres

# Clean up
docker rm -f $(docker ps -aq)
```

### Tests Fail Intermittently
- **Cause:** Parallel test execution with shared state
- **Fix:** Use `TestDataBuilder::from_test_name()` for unique data
- **Or:** Run sequentially: `cargo test -- --test-threads=1`

### Slow Tests
```bash
# Profile tests
cargo test -- --show-output

# Run only fast tests
cargo test --lib  # Unit tests only
```

### Database Connection Errors
- **Cause:** Docker not running or testcontainers can't start
- **Fix:** Ensure Docker is running: `docker ps`

---

## CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Unit tests (fast)
      - name: Unit Tests
        run: cargo test --lib --workspace

      # Integration tests (medium)
      - name: Integration Tests
        run: cargo test --workspace --test integration_test

      # Handler tests (medium)
      - name: Handler Tests
        run: cargo test --workspace --test handler_test

      # E2E tests (slow, run last)
      - name: E2E Tests
        run: cargo test -p zerg_api --test e2e_test
```

---

## Summary

| Type | Location | Speed | Scope | Use For |
|------|----------|-------|-------|---------|
| **Unit** | `src/*.rs` | 0.00s | Function | Business logic |
| **Integration** | `tests/integration_test.rs` | ~10s | Service → DB | DB operations |
| **Handler** | `tests/handler_test.rs` | ~2-5s | HTTP handlers | API contracts |
| **E2E** | `apps/*/tests/e2e_test.rs` | ~15-30s | Full app | User journeys |

**Follow the pyramid:** Many unit tests, some integration/handler tests, few E2E tests.

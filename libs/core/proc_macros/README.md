# Proc Macros

Derive macros for reducing boilerplate across domain entities and API resources.

## `#[derive(ApiResource)]`

**Crate:** `api_resource`

Implements the `ApiResource` trait on a struct, providing REST API metadata constants.

### Input

```rust
#[derive(ApiResource)]
struct User {
    id: Uuid,
    email: String,
}
```

### Generated

```rust
impl ApiResource for User {
    const URL: &'static str = "/user";
    const COLLECTION: &'static str = "users";   // auto-pluralized
    const TAG: &'static str = "Users";           // auto-pluralized
}
```

### Custom Overrides

```rust
#[derive(ApiResource)]
#[api_resource(collection = "people", url = "/person", tag = "People")]
struct Person { /* ... */ }
```

Pluralization uses the `pluralizer` crate (e.g., `Story` → `stories`).

---

## `#[derive(SeaOrmResource)]`

**Crate:** `sea_orm_resource`

Like `ApiResource` but extracts the base name from `#[sea_orm(table_name = "...")]`.

### Input

```rust
#[derive(DeriveEntityModel, SeaOrmResource)]
#[sea_orm(table_name = "cloud_resources")]
struct Model {
    id: Uuid,
    name: String,
}
```

### Generated

```rust
impl ApiResource for Model {
    const URL: &'static str = "/cloud-resources";      // underscores → hyphens
    const COLLECTION: &'static str = "cloud_resources"; // from table_name
    const TAG: &'static str = "Cloud Resources";        // snake_case → Title Case
}
```

### Custom Overrides

```rust
#[derive(SeaOrmResource)]
#[sea_orm(table_name = "cloud_resources")]
#[sea_orm_resource(url = "/resources", tag = "Resources")]
struct Model { /* ... */ }
```

Requires `#[sea_orm(table_name = "...")]` — compilation fails without it.

---

## `#[derive(SelectableFields)]`

**Crate:** `selectable_fields`

Implements the `field_selector::SelectableFields` trait for role-based field access control.

### Input

```rust
#[derive(SelectableFields)]
struct User {
    pub name: String,
    #[field(role = "admin")]
    pub email: String,
    #[field(skip)]
    pub password_hash: String,
    #[field(rename = "display_name")]
    pub internal_name: String,
}
```

### Generated

```rust
impl SelectableFields for User {
    fn available_fields() -> Vec<&'static str> {
        vec!["name", "email", "display_name"]  // skipped fields excluded
    }

    fn restricted_fields() -> Vec<&'static str> {
        vec!["password_hash"]
    }

    fn field_access() -> Vec<FieldAccess> {
        vec![
            FieldAccess { name: "name", min_role: Role::Anonymous },
            FieldAccess { name: "email", min_role: Role::Admin },
            FieldAccess { name: "display_name", min_role: Role::Anonymous },
        ]
    }
}
```

### Field Attributes

| Attribute | Effect |
|-----------|--------|
| `#[field(skip)]` | Hides field from API, adds to `restricted_fields()` |
| `#[field(role = "admin")]` | Requires minimum role (`anonymous`, `user`, `admin`) |
| `#[field(rename = "x")]` | Exposes field under a different name |

Attributes can be combined: `#[field(role = "user", rename = "display")]`.

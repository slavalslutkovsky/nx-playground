# Database Migrations

SeaORM database migrations for the nx-playground project.

## Quick Start

### Prerequisites

1. **Install sea-orm-cli:**
   ```bash
   cargo install sea-orm-cli
   ```

2. **Setup direnv** (see main [README.md](../../README.md#environment-variables)):
   ```bash
   # Install direnv
   brew install direnv  # macOS

   # Add to shell
   eval "$(direnv hook zsh)" >> ~/.zshrc
   source ~/.zshrc

   # Setup environment
   cp ../../.env.example ../../.env
   vim ../../.env  # Edit with your database credentials
   direnv allow
   ```

### Run Migrations

```bash
# With direnv (recommended) - just run commands!
sea-orm-cli migrate up
sea-orm-cli migrate status
sea-orm-cli migrate down

# Without direnv - specify database and migration directory
DATABASE_URL=postgres://myuser:mypassword@localhost/mydatabase \
  sea-orm-cli migrate -d libs/migration up
```

## Migration Commands

All commands shown with direnv (recommended). For without direnv, add `-d libs/migration` flag.

### Basic Operations

```bash
# Run all pending migrations
sea-orm-cli migrate up

# Rollback last migration
sea-orm-cli migrate down

# Rollback last 3 migrations
sea-orm-cli migrate down 3

# Check migration status
sea-orm-cli migrate status
```

### Development Commands

```bash
# Drop all tables and re-run migrations (DESTRUCTIVE - dev only!)
sea-orm-cli migrate fresh

# Rollback all migrations
sea-orm-cli migrate reset

# Rollback and re-run all migrations
sea-orm-cli migrate refresh
```

## Creating New Migrations

```bash
# Generate new migration file
sea-orm-cli migrate generate <migration_name>

# Examples:
sea-orm-cli migrate generate create_users_table
sea-orm-cli migrate generate add_email_to_projects
sea-orm-cli migrate generate add_index_to_projects
```

This creates a new file: `libs/migration/src/m{timestamp}_{name}.rs`

### Migration Template

Edit the generated file to define your schema changes:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(pk_uuid(Users::Id))
                    .col(string(Users::Email))
                    .col(string(Users::Name))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Name,
}
```

After creating the migration:
```bash
sea-orm-cli migrate up
```

## Current Migrations

- `m20241129_000001_create_projects.rs` - Projects table with cloud provider, environment, status
- `m20241129_000002_create_cloud_resources.rs` - Cloud resources table with FK to projects

## Generating Entities

Auto-generate Rust entities from your database schema:

```bash
# Generate entities for all tables
sea-orm-cli generate entity \
  -o libs/domains/projects/src/entity

# For specific tables only
sea-orm-cli generate entity \
  -o libs/domains/projects/src/entity \
  --tables projects,cloud_resources

# With expanded format (more readable)
sea-orm-cli generate entity \
  -o libs/domains/projects/src/entity \
  --expanded-format
```

> **Note:** With direnv, `DATABASE_URL` is automatically available. Without direnv, add `-u postgres://user:pass@host/db` flag.

## Production Deployment

**Important:** Run migrations **before** deploying your application!

```bash
# In CI/CD pipeline
export DATABASE_URL=<production-database-url>
export MIGRATION_DIR=libs/migration
sea-orm-cli migrate up

# Then deploy the application
cargo run -p zerg_api
```

## Environment Variables (via direnv)

When you `cd` into the project with direnv setup, these are automatically loaded from `.env`:

```bash
DATABASE_URL=postgres://username:password@localhost/database
```

And these are set by `.envrc`:

```bash
MIGRATION_DIR=libs/migration    # Tells sea-orm-cli where migrations are
DATABASE_SCHEMA=public           # PostgreSQL schema
```

## Without direnv

If you're not using direnv, you need to specify options manually:

```bash
# Run migrations
DATABASE_URL=postgres://user:pass@localhost/db \
  sea-orm-cli migrate -d libs/migration up

# Generate entities
sea-orm-cli generate entity \
  -u postgres://user:pass@localhost/db \
  -o libs/domains/projects/src/entity
```

## Migration Binary

This crate includes a binary (`src/main.rs`) that `sea-orm-cli` uses internally:

```rust
use migration::Migrator;
use sea_orm_migration::cli;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
```

You typically don't run this directly - `sea-orm-cli` handles it.

## Best Practices

### Development
- ✅ Use direnv for automatic environment loading
- ✅ Use `sea-orm-cli migrate fresh` to reset local database
- ✅ Always test both `up` and `down` migrations
- ✅ Keep migrations small and focused on one change
- ✅ Use descriptive migration names

### Production
- ✅ Run migrations in a separate step before deployment
- ✅ Test migrations in staging environment first
- ✅ Keep database backups before running migrations
- ✅ Use separate database user for migrations vs application
- ❌ Never use auto-migration in production
- ❌ Never use `fresh` or `reset` commands in production

## Troubleshooting

### "Environment variable 'DATABASE_URL' not set"

**With direnv:**
```bash
# Check if direnv is active
echo $DIRENV_DIR  # Should show current directory

# If not, allow direnv
direnv allow

# Check .env file exists
cat .env  # Should show DATABASE_URL
```

**Without direnv:**
```bash
export DATABASE_URL=postgres://myuser:mypassword@localhost/mydatabase
```

### "manifest path does not exist"

**With direnv:**
```bash
# Check MIGRATION_DIR is set
echo $MIGRATION_DIR  # Should show: libs/migration

# If not, reload direnv
direnv allow
```

**Without direnv:**
```bash
# Add -d flag
sea-orm-cli migrate -d libs/migration up
```

### Check Migration Status

```bash
# Using sea-orm-cli (with direnv)
sea-orm-cli migrate status

# Or query database directly
psql $DATABASE_URL -c "SELECT * FROM seaql_migrations ORDER BY version;"
```

### Reset Stuck Migration

If a migration partially fails:

```bash
# Development: Drop and recreate
sea-orm-cli migrate fresh

# Production: Manually fix database state, then:
# 1. Check what's applied
psql $DATABASE_URL -c "SELECT * FROM seaql_migrations;"

# 2. Manually delete failed migration record if needed
psql $DATABASE_URL \
  -c "DELETE FROM seaql_migrations WHERE version = 'm20241129_000003';"

# 3. Re-run migration
sea-orm-cli migrate up
```

## See Also

- [Project Setup](../../README.md#project-setup) - Environment and direnv configuration
- [SeaORM Migration Docs](https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/)
- [SeaORM CLI Docs](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/)

# Database Migrations Guide

This guide covers database migrations using [Atlas](https://atlasgo.io/) for both local development and Kubernetes deployments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Directory Structure](#directory-structure)
- [Local Development](#local-development)
  - [Quick Start](#quick-start)
  - [Creating New Migrations](#creating-new-migrations)
  - [Editing Existing Migrations](#editing-existing-migrations)
  - [Inspecting the Database](#inspecting-the-database)
- [Kubernetes (Kind + CNPG)](#kubernetes-kind--cnpg)
  - [Prerequisites](#kubernetes-prerequisites)
  - [Deploying CNPG Cluster](#deploying-cnpg-cluster)
  - [Applying Migrations](#applying-migrations-in-kubernetes)
  - [Updating Migrations](#updating-migrations-in-kubernetes)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Local Development

```bash
# Atlas CLI (migrations tool)
curl -sSf https://atlasgo.sh | sh

# PostgreSQL client (for psql)
brew install postgresql  # macOS
# or: apt install postgresql-client  # Ubuntu

# Docker (for containers)
brew install --cask docker
```

### Kubernetes

```bash
# kubectl and Kind
brew install kubectl kind

# Helm (for Atlas operator)
brew install helm
```

---

## Directory Structure

```
manifests/
├── schemas/
│   └── atlas.hcl              # Atlas project config (environments)
│
├── migrations/
│   └── mydatabase/
│       ├── 20240204000001_initial.sql         # Schema migration
│       ├── 20240204000002_seed_data.sql       # Seed data
│       ├── 20240205000001_*.sql               # Additional migrations
│       └── atlas.sum                          # Checksums (auto-generated)
│
└── cnpg/
    ├── base/
    │   ├── cluster.yaml           # CNPG PostgreSQL cluster
    │   ├── atlas-migration.yaml   # AtlasMigration CRD
    │   └── migrations-configmap.yaml  # Migrations as ConfigMap
    └── overlays/
        ├── dev/
        ├── staging/
        └── prod/
```

**Important:** SQL migration files are the **single source of truth**. No HCL schema files for tables - we use versioned SQL migrations directly.

---

## Local Development

### Quick Start

```bash
# Option 1: Full reset (recommended for clean start)
just reset-db

# Option 2: Step by step
just _docker-up          # Start PostgreSQL container
just db-create           # Create mydatabase
just migrate             # Apply migrations
```

### Common Commands

#### Basic Commands
| Command | Description |
|---------|-------------|
| `just migrate` | Apply pending migrations |
| `just migrate-status` | Check migration status |
| `just migrate-dry` | Preview migrations (dry run) |
| `just migrate-lint` | Lint migrations for issues |
| `just migrate-hash` | Regenerate checksums |

#### Development Commands (UNSAFE for production)
| Command | Description |
|---------|-------------|
| `just dev-migrate-refresh` | Re-hash and apply (after edits) |
| `just dev-migrate-refresh fresh` | Drop DB + re-hash + apply |
| `just db-reset` | Drop, recreate, and migrate |
| `just reset-db` | Full reset (docker + db + migrate) |

#### Production Commands (SAFE)
| Command | Description |
|---------|-------------|
| `just prod-migrate` | Full workflow: lint → status → dry-run → confirm → apply |
| `just prod-migrate-preview` | Preview only (no changes) |

#### Inspection Commands
| Command | Description |
|---------|-------------|
| `just db-inspect` | Show current schema (HCL) |
| `just db-inspect-sql` | Show current schema (SQL) |
| `just db-drift` | Check for schema drift |

### Creating New Migrations

1. **Create a new SQL file with timestamp prefix:**

   ```bash
   # Generate timestamp-prefixed filename
   touch manifests/migrations/mydatabase/$(date +%Y%m%d%H%M%S)_description.sql
   ```

2. **Write your migration SQL:**

   ```sql
   -- Add a new column
   ALTER TABLE users ADD COLUMN phone VARCHAR(20);

   -- Create index
   CREATE INDEX idx_users_phone ON users(phone);
   ```

3. **Regenerate checksums:**

   ```bash
   just migrate-hash
   ```

4. **Apply the migration:**

   ```bash
   just migrate
   ```

### Editing Existing Migrations

When you edit an existing migration file, Atlas detects the checksum mismatch. Here's the workflow:

#### Development: Re-hash and apply

```bash
# After editing migration files (keeps existing data)
just dev-migrate-refresh

# Or drop database and start fresh (loses all data)
just dev-migrate-refresh fresh
```

#### Production: You should NOT edit migrations

In production, migrations should be immutable once applied. If you need changes:

1. **Create a new migration** with the fix/change
2. **Test in dev/staging** before production
3. **Use the safe workflow:**

```bash
# Preview what will happen (no changes)
just prod-migrate-preview

# Full workflow with confirmation prompt
just prod-migrate
```

#### Manual Steps (if needed)

```bash
# 1. Edit your migration file(s)
vim manifests/migrations/mydatabase/20240204000001_initial.sql

# 2. Re-hash to update checksums
just migrate-hash

# 3. Apply (may fail if changes conflict with existing data)
just migrate

# 4. If conflicts, drop and recreate (DEV ONLY)
just db-reset
```

### Inspecting the Database

```bash
# View current schema as HCL
just db-inspect

# View current schema as SQL
just db-inspect-sql

# Check for drift (diff between migrations and live DB)
just db-drift

# Generate ERD diagram (Mermaid)
just db-erd
```

---

## Kubernetes (Kind + CNPG)

### Quick Start (Full Setup)

For a fresh Kind cluster with everything configured:

```bash
# One command to rule them all
just kind-cnpg-setup

# Or for staging environment
just kind-cnpg-setup staging
```

This will:
1. Create Kind cluster (if not exists)
2. Install CNPG operator
3. Install Atlas operator
4. Deploy CNPG PostgreSQL cluster
5. Apply migrations via Atlas operator

### Manual Setup (Step by Step)

#### 1. Create Kind Cluster

```bash
# Using the nu script (recommended - includes ingress, namespaces)
just local-up --skip-dbs --skip-tilt

# Or basic Kind cluster
kind create cluster --name dev
```

#### 2. Install Operators

```bash
# Install both operators
just operators-install

# Or individually:
just cnpg-install         # CNPG operator
just atlas-operator-install  # Atlas operator
```

#### 3. Deploy CNPG Cluster

```bash
# Dev environment
just cnpg-dev

# Staging environment
just cnpg-staging

# Production environment
just cnpg-prod
```

### Deploying CNPG Cluster

The CNPG cluster creates a PostgreSQL instance with automatic failover, backups, and TLS.

```bash
# Deploy to dev environment
just cnpg-dev

# Or for other environments:
just cnpg-staging
just cnpg-prod
```

This deploys:
- **Namespace:** `mydatabase-dev` (or staging/prod)
- **CNPG Cluster:** PostgreSQL 18 with configured resources
- **Credentials Secret:** Auto-generated by kustomize (dev) or external-secrets (prod)
- **AtlasMigration:** Watches for migration ConfigMap changes
- **Migrations ConfigMap:** SQL files bundled for the operator

### Applying Migrations in Kubernetes

Migrations are applied automatically by the Atlas Operator when you deploy:

```bash
# Deploy (includes migrations)
just cnpg-dev

# Check migration status
kubectl get atlasmigration -n mydatabase-dev -o wide

# View migration logs
kubectl logs -n mydatabase-dev -l app.kubernetes.io/name=mydatabase-migration
```

The Atlas Operator:
1. Watches the `mydatabase-migrations` ConfigMap
2. Connects to CNPG using the auto-generated `mydatabase-db-app` secret
3. Applies pending migrations
4. Updates status on the AtlasMigration resource

### Updating Migrations in Kubernetes

When you change migration files locally:

1. **Edit your migration files locally:**

   ```bash
   vim manifests/migrations/mydatabase/20240204000001_initial.sql
   ```

2. **Re-hash the migrations:**

   ```bash
   just migrate-hash
   ```

3. **Regenerate the ConfigMap and redeploy:**

   ```bash
   just cnpg-dev
   ```

   This runs `just cnpg-gen-migrations` internally, which:
   - Reads all SQL files from `manifests/migrations/mydatabase/`
   - Generates `manifests/cnpg/base/migrations-configmap.yaml`
   - Includes the `atlas.sum` file

4. **Verify the migration applied:**

   ```bash
   just cnpg-status
   # Or:
   kubectl get atlasmigration -n mydatabase-dev -o yaml
   ```

### Manual ConfigMap Generation

If you need to regenerate the ConfigMap without deploying:

```bash
just cnpg-gen-migrations
```

This creates/updates `manifests/cnpg/base/migrations-configmap.yaml`.

### Port-Forward to Access Database

```bash
# Get the service name
kubectl get svc -n mydatabase-dev

# Port-forward to the read-write service
kubectl port-forward -n mydatabase-dev svc/dev-mydatabase-db-rw 5433:5432

# Connect with psql
psql "postgres://mydatabase:dev-password-change-me@localhost:5433/mydatabase"
```

### Cluster Operations

```bash
# Check cluster status
just cnpg-status

# View cluster logs
just cnpg-logs

# Apply migrations to cluster (via kompose)
just migrate-cluster
```

---

## Troubleshooting

### Checksum Mismatch Error

```
You have a checksum error in your migration directory.
    L4: 20240205000001_add_project_repository.sql was edited
Please check your migration files and run 'atlas migrate hash'
```

**Solution (Development):** Re-hash after editing:

```bash
just dev-migrate-refresh        # Re-hash and apply
# or
just dev-migrate-refresh fresh  # Drop DB and start fresh
```

**Solution (Production):** This should NOT happen in production. If it does:
1. Investigate why the file changed (corruption? manual edit?)
2. Restore the original file from git
3. Create a new migration for the intended change

### Migration Already Applied

If Atlas says a migration was already applied but you want to re-run:

```bash
# Option 1: Reset the database (loses data)
just db-reset

# Option 2: Mark as baseline and continue
atlas migrate apply --dir "file://manifests/migrations/mydatabase" \
  --url "postgres://myuser:mypassword@localhost:5432/mydatabase?sslmode=disable" \
  --baseline "20240204000001"
```

### CNPG Pod Not Starting

```bash
# Check pod status
kubectl get pods -n mydatabase-dev -l cnpg.io/cluster

# Check events
kubectl get events -n mydatabase-dev --sort-by='.lastTimestamp'

# Check operator logs
kubectl logs -n cnpg-system deployment/cnpg-controller-manager
```

### Atlas Operator Not Applying Migrations

```bash
# Check AtlasMigration status
kubectl describe atlasmigration -n mydatabase-dev

# Check operator logs
kubectl logs -n atlas-operator deployment/atlas-operator

# Verify ConfigMap exists
kubectl get configmap mydatabase-migrations -n mydatabase-dev -o yaml
```

### Connection Refused

Ensure PostgreSQL is running:

```bash
# Local
docker ps | grep postgres

# Kubernetes
kubectl get pods -n mydatabase-dev -l cnpg.io/cluster
```

---

## Environment Reference

| Environment | URL | Port | Usage |
|-------------|-----|------|-------|
| Local | `localhost:5432` | 5432 | Docker Compose |
| Cluster | `localhost:5433` | 5433 | Kind (kompose) |
| CNPG Dev | `dev-mydatabase-db-rw:5432` | 5432 | In-cluster |
| Production | `DATABASE_URL` env | varies | Cloud/Production |

### Atlas Config Environments

The `manifests/schemas/atlas.hcl` defines these environments:

```bash
# Use local environment
atlas migrate apply --env local

# Use cluster environment
atlas migrate apply --env cluster

# Use CNPG environment (requires DATABASE_URL)
DATABASE_URL="postgres://..." atlas migrate apply --env cnpg
```

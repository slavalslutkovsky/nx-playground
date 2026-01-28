# Platform APIs - Multi-Cloud Crossplane Module
### see examples of kcl at [link](https://github.com/yurikrupnik/mussia46/blob/c63ab373b0fdf1f0f8cd71ec80f0782ae133ef0c/generations/kcl/my_package/new-storage.yaml)
### see examples of upbound cli at [link](https://docs.upbound.io/getstarted/introduction/build-and-push/)
This project provides multi-cloud Crossplane APIs using KCL (Kcl Configuration Language) for type-safe composition logic.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              XApplication (parent XRD)                       │
│              platform.io/v1alpha1                            │
└─────────────────────────────────────────────────────────────┘
                            │
                     Composition
                     (compose-application)
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
   ┌─────────────────┐             ┌─────────────────┐
   │     XBucket     │             │    XDatabase    │
   │ (child XRD)     │             │   (child XRD)   │
   └─────────────────┘             └─────────────────┘
            │                               │
     Composition                     Composition
     (compose-bucket)                (compose-database)
            │                               │
   ┌────────┴────────┐             ┌────────┴────────┐
   │ if aws:         │             │ if aws:         │
   │   S3 Bucket     │             │   RDS Instance  │
   │ elif gcp:       │             │ elif gcp:       │
   │   GCS Bucket    │             │   CloudSQL      │
   └─────────────────┘             └─────────────────┘
```

## Available APIs

### XBucket / Bucket (storage.platform.io/v1alpha1)

Cloud storage abstraction supporting AWS S3 and GCP GCS.

```yaml
apiVersion: storage.platform.io/v1alpha1
kind: Bucket
metadata:
  name: my-bucket
  namespace: default
spec:
  provider: gcp          # aws | gcp
  region: us-east1
  name: globally-unique-bucket-name
  environment: dev       # dev | staging | prod
  storageClass: standard # standard | nearline | coldline
  versioning: false
  encryption: true
  publicAccess: false
  deletionPolicy: Delete # Delete | Orphan
```

### XDatabase / Database (database.platform.io/v1alpha1)

Managed database abstraction supporting AWS RDS, GCP CloudSQL, and CloudNative-PG (for local/K8s-native).

**Cloud provider (AWS/GCP):**
```yaml
apiVersion: database.platform.io/v1alpha1
kind: Database
metadata:
  name: my-database
  namespace: default
spec:
  provider: gcp          # aws | gcp | kubernetes
  region: us-east1       # required for aws/gcp
  engine: postgres       # postgres | mysql
  engineVersion: "15"
  size: small            # small | medium | large
  storageGB: 20
  environment: dev
  deletionPolicy: Delete
```

**Local development (CloudNative-PG):**
```yaml
apiVersion: database.platform.io/v1alpha1
kind: Database
metadata:
  name: my-local-db
  namespace: default
spec:
  provider: kubernetes   # Uses CloudNative-PG operator
  engine: postgres       # CNPG only supports postgres
  engineVersion: "16"
  size: small
  storageGB: 10
  instances: 1           # Number of replicas (CNPG only)
  namespace: default     # Target namespace (CNPG only)
  environment: dev
```

### XRegistry / Registry (registry.platform.io/v1alpha1)

Container registry abstraction supporting AWS ECR, GCP Artifact Registry, and Harbor (for local/K8s-native).

**Cloud provider (AWS ECR):**
```yaml
apiVersion: registry.platform.io/v1alpha1
kind: Registry
metadata:
  name: my-ecr
  namespace: default
spec:
  provider: aws
  name: my-app-images
  region: us-east-1
  immutableTags: true
  scanOnPush: true
  retentionDays: 30
  environment: dev
```

**Cloud provider (GCP Artifact Registry):**
```yaml
apiVersion: registry.platform.io/v1alpha1
kind: Registry
metadata:
  name: my-artifact-registry
  namespace: default
spec:
  provider: gcp
  name: my-app-images
  region: us-east1
  format: docker       # docker | npm | maven | python
  environment: dev
```

**Local development (Harbor):**
```yaml
apiVersion: registry.platform.io/v1alpha1
kind: Registry
metadata:
  name: my-local-registry
  namespace: default
spec:
  provider: kubernetes  # Uses Harbor operator (CNCF Graduated)
  name: local-registry
  namespace: default
  environment: dev
```

### XApplication / Application (platform.io/v1alpha1)

Full application stack that composes XBucket and XDatabase.

```yaml
apiVersion: platform.io/v1alpha1
kind: Application
metadata:
  name: my-app
  namespace: default
spec:
  name: my-full-app
  provider: gcp
  region: us-east1
  environment: dev
  storage:
    enabled: true
    storageClass: standard
  database:
    enabled: true
    engine: postgres
    size: small
```

## Project Structure

```
platform/
├── upbound.yaml                    # Project manifest
├── schemas/                        # Source of truth (published to OCI)
│   ├── common.k                    # Shared types, OpenAPI helpers
│   ├── helpers.k                   # Composition helpers (metadata, tags)
│   ├── mappings.k                  # Provider mappings (sizes, regions)
│   ├── regions.k                   # Region mappings (GCP <-> AWS)
│   ├── xrd.k                       # XRD generator
│   └── {resource}.k                # Resource schemas + XRD metadata
├── apis/
│   ├── xbuckets/
│   │   ├── definition.yaml         # XRD (generated from render/)
│   │   └── composition.yaml        # Composition
│   ├── xdatabases/
│   ├── xnetworks/
│   ├── xregistries/
│   └── xapplications/
├── functions/
│   ├── compose-bucket/main.k       # Storage KCL logic
│   ├── compose-database/main.k     # Database KCL logic
│   ├── compose-network/main.k      # Network KCL logic
│   ├── compose-registry/main.k     # Registry KCL logic
│   └── compose-application/main.k  # Application KCL logic
├── render/                         # XRD generators
│   └── {resource}_xrd.k
├── examples/                       # Example claims
├── tests/                          # Composition tests
│   └── test-x{resource}/
└── .up/kcl/models/                 # Auto-generated provider schemas
```

## Schemas Package

Schemas are published to OCI registry for use by composition functions:

```
docker.io/yurikrupnik/platform-schemas:0.1.0
```

Functions reference schemas via:
```toml
[dependencies]
schemas = { oci = "oci://docker.io/yurikrupnik/platform-schemas", tag = "0.1.0" }
```

### Generate XRDs from schemas

```bash
kcl run render/bucket_xrd.k > apis/xbuckets/definition.yaml
kcl run render/network_xrd.k > apis/xnetworks/definition.yaml
```

### Publish schemas

```bash
cd schemas && kcl mod push oci://docker.io/yurikrupnik/platform-schemas
```

## Development Commands

```bash
# Build the project
up project build

# Run locally (creates KIND cluster)
up project run --local --ingress

# Run composition tests
up test run tests/*

# Render a composition (preview output)
up composition render apis/xbuckets/composition.yaml \
  --composite-resource examples/bucket-gcp.yaml

# Push to Docker Hub
up project push docker.io/yurikrupnik/platform:v0.1.0
```

## Installation (existing cluster)

```bash
# Install dependencies (providers + functions)
kubectl apply -f - <<EOF
apiVersion: pkg.crossplane.io/v1
kind: Provider
metadata:
  name: upbound-provider-family-gcp
spec:
  package: xpkg.upbound.io/upbound/provider-family-gcp:v2.4.0
---
apiVersion: pkg.crossplane.io/v1
kind: Provider
metadata:
  name: upbound-provider-family-aws
spec:
  package: xpkg.upbound.io/upbound/provider-family-aws:v1.20.0
---
apiVersion: pkg.crossplane.io/v1
kind: Function
metadata:
  name: function-kcl
spec:
  package: xpkg.upbound.io/crossplane-contrib/function-kcl:v0.12.0
---
apiVersion: pkg.crossplane.io/v1beta1
kind: Function
metadata:
  name: function-auto-ready
spec:
  package: xpkg.upbound.io/crossplane-contrib/function-auto-ready:v0.3.0
EOF

# Apply XRDs
kubectl apply -f apis/xbuckets/definition.yaml
kubectl apply -f apis/xdatabases/definition.yaml
kubectl apply -f apis/xapplications/definition.yaml

# Apply Compositions
kubectl apply -f apis/xbuckets/composition.yaml
kubectl apply -f apis/xdatabases/composition.yaml
kubectl apply -f apis/xapplications/composition.yaml
```

## Examples

Create a GCP bucket:

```bash
kubectl apply -f examples/bucket-gcp.yaml
```

Create a full application stack:

```bash
kubectl apply -f examples/application-full.yaml
```

## Provider Configuration

Before creating resources, configure cloud provider credentials:

### GCP

```yaml
apiVersion: gcp.upbound.io/v1beta1
kind: ProviderConfig
metadata:
  name: default
spec:
  projectID: your-project-id
  credentials:
    source: Secret
    secretRef:
      namespace: crossplane-system
      name: gcp-credentials
      key: credentials
```

### AWS

```yaml
apiVersion: aws.upbound.io/v1beta1
kind: ProviderConfig
metadata:
  name: default
spec:
  credentials:
    source: Secret
    secretRef:
      namespace: crossplane-system
      name: aws-credentials
      key: credentials
```

## Testing

Run composition tests:

```bash
up test run tests/*
```

Render compositions locally:

```bash
up composition render apis/xbuckets/composition.yaml \
  --composite-resource examples/bucket-gcp.yaml
```

## Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| provider-family-gcp | v2.4.0 | GCP resources |
| provider-family-aws | v1.20.0 | AWS resources |
| provider-gcp-storage | v2.4.0 | GCS buckets |
| provider-gcp-sql | v2.4.0 | CloudSQL |
| provider-aws-s3 | v1.20.0 | S3 buckets |
| provider-aws-rds | v1.20.0 | RDS instances |
| provider-aws-ecr | v1.20.0 | ECR repositories |
| provider-gcp-cloudplatform | v2.4.0 | Artifact Registry |
| function-kcl | v0.12.0 | KCL composition functions |
| function-auto-ready | v0.3.0 | Auto-ready detection |
| cloudnative-pg (KCL) | v1.27.0 | CloudNative-PG schemas |
| harbor-operator (KCL) | v0.2.1 | Harbor registry schemas |

## Instance Size Mapping

### Database Sizes

| Size | AWS RDS | GCP CloudSQL | CloudNative-PG |
|------|---------|--------------|----------------|
| small | db.t3.micro | db-f1-micro | 256Mi/100m - 512Mi/500m |
| medium | db.t3.small | db-g1-small | 512Mi/250m - 1Gi/1000m |
| large | db.t3.medium | db-custom-2-4096 | 1Gi/500m - 2Gi/2000m |

## Local Development (Kind)

For local development in Kind clusters, use `provider: kubernetes` which creates CloudNative-PG clusters.

### Prerequisites

Install the CloudNative-PG operator:

```bash
kubectl apply -f https://github.com/cloudnative-pg/cloudnative-pg/releases/download/v1.27.0/cnpg-1.27.0.yaml
```

### Example

```bash
kubectl apply -f examples/database-local.yaml
```

This creates an in-cluster PostgreSQL database managed by CNPG - no cloud credentials needed.

## Naming Conventions

- **XRD names**: `x<resource>s.<group>` (e.g., `xbuckets.storage.platform.io`)
- **Composite kinds**: `X<Resource>` (e.g., `XBucket`)
- **Claim kinds**: `<Resource>` (e.g., `Bucket`)
- **Composition names**: `x<resource>` (e.g., `xbucket`)

The `X` prefix follows Crossplane convention to distinguish composite resources from claims.

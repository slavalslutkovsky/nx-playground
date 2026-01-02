# Pulumi GCP Organization Infrastructure Structure

## Overview

This document outlines the recommended Pulumi project structure for managing the `yurikrupnik.com` GCP organization. The structure follows infrastructure-as-code best practices: separation of concerns, minimal blast radius, and clear ownership boundaries.

## Design Principles

1. **State Isolation** - Separate Pulumi projects for different lifecycle and risk levels
2. **Blast Radius Minimization** - Changes to one area don't affect others
3. **Clear Ownership** - Each project has a defined purpose and owner
4. **Environment Parity** - Consistent patterns across dev/staging/prod
5. **Least Privilege** - Service accounts scoped to their specific project needs

---

## Recommended Project Structure

```
infrastructure/
├── 01-bootstrap/              # One-time org setup (run manually, rarely changes)
├── 02-organization/           # Org policies, folders, billing
├── 03-iam/                    # Organization & folder-level IAM
├── 04-networking/             # Shared VPC, DNS, interconnects
├── 05-shared-services/        # Artifact Registry, Secret Manager, KMS
├── environments/
│   ├── dev/                   # Dev environment resources
│   ├── staging/               # Staging environment resources
│   └── prod/                  # Production environment resources
└── workloads/
    ├── data/                  # Data platform resources
    ├── machine-learning/      # ML platform resources
    ├── mlops/                 # MLOps pipeline resources
    └── platform/              # Platform team resources
```

---

## Project Details

### 1. Bootstrap (`01-bootstrap/`)

**Purpose:** Initial setup that must exist before other Pulumi projects can run. Run once manually by an org admin.

**Manages:**
- Pulumi state bucket (GCS)
- Terraform/Pulumi service account for automation
- Initial IAM for the automation service account
- Enable required APIs at org level

**State:** Local or separate backend (chicken-and-egg problem)

**Run frequency:** Once, or when adding new automation capabilities

```
01-bootstrap/
├── Pulumi.yaml
├── Pulumi.prod.yaml
├── index.ts
└── README.md
```

**Resources created:**
```typescript
// GCS bucket for Pulumi state
const stateBucket = new gcp.storage.Bucket("pulumi-state", {
    name: "yurikrupnik-pulumi-state",
    location: "EU",
    versioning: { enabled: true },
    uniformBucketLevelAccess: true,
});

// Service account for Pulumi automation
const pulumiSA = new gcp.serviceaccount.Account("pulumi-automation", {
    accountId: "pulumi-automation",
    displayName: "Pulumi Automation Service Account",
});

// Grant org-level permissions to automation SA
const orgAdminBinding = new gcp.organizations.IAMMember("pulumi-org-admin", {
    orgId: "477132171939",
    role: "roles/resourcemanager.organizationAdmin",
    member: pulumi.interpolate`serviceAccount:${pulumiSA.email}`,
});
```

---

### 2. Organization (`02-organization/`)

**Purpose:** Manage the organizational hierarchy and policies. Changes here affect the entire org.

**Manages:**
- Folder structure
- Organization policies (constraints)
- Billing account associations
- Audit logging configuration

**Dependencies:** Bootstrap

**Run frequency:** Rarely (when org structure changes)

```
02-organization/
├── Pulumi.yaml
├── Pulumi.prod.yaml
├── src/
│   ├── index.ts
│   ├── folders.ts
│   ├── org-policies.ts
│   ├── billing.ts
│   └── audit-logging.ts
└── README.md
```

**Resources created:**
```typescript
// Folder structure matching your org
const folders = {
    data: new gcp.organizations.Folder("data", {
        displayName: "data",
        parent: `organizations/${orgId}`,
    }),
    dev: new gcp.organizations.Folder("dev", {
        displayName: "dev",
        parent: `organizations/${orgId}`,
    }),
    machineLearning: new gcp.organizations.Folder("machine-learning", {
        displayName: "machine-learning",
        parent: `organizations/${orgId}`,
    }),
    marketing: new gcp.organizations.Folder("marketing", {
        displayName: "marketing",
        parent: `organizations/${orgId}`,
    }),
    mlops: new gcp.organizations.Folder("mlops", {
        displayName: "mlops",
        parent: `organizations/${orgId}`,
    }),
    platform: new gcp.organizations.Folder("platform", {
        displayName: "platform",
        parent: `organizations/${orgId}`,
    }),
};

// Organization policies
const requireOsLogin = new gcp.orgpolicy.Policy("require-os-login", {
    name: `organizations/${orgId}/policies/compute.requireOsLogin`,
    parent: `organizations/${orgId}`,
    spec: {
        rules: [{ enforce: "TRUE" }],
    },
});

const restrictPublicIp = new gcp.orgpolicy.Policy("restrict-public-ip", {
    name: `organizations/${orgId}/policies/compute.vmExternalIpAccess`,
    parent: `organizations/${orgId}`,
    spec: {
        rules: [{ denyAll: "TRUE" }],
    },
});
```

---

### 3. IAM (`03-iam/`)

**Purpose:** Centralized identity and access management. All user/group permissions defined here.

**Manages:**
- Organization-level IAM bindings
- Folder-level IAM bindings
- Google Groups for role-based access
- Service accounts for cross-project use
- Workload Identity configurations

**Dependencies:** Organization (needs folder IDs)

**Run frequency:** When team membership or permissions change

```
03-iam/
├── Pulumi.yaml
├── Pulumi.prod.yaml
├── src/
│   ├── index.ts
│   ├── org-iam.ts           # Org-level bindings
│   ├── folder-iam.ts        # Folder-level bindings
│   ├── groups.ts            # Google Groups definitions
│   ├── service-accounts.ts  # Shared service accounts
│   └── workload-identity.ts # GKE workload identity
└── README.md
```

**Resources created:**
```typescript
// Organization admins
const orgAdmins = [
    "user:yurik@yurikrupnik.com",
    "user:slava@yurikrupnik.com",
];

const orgAdminBindings = orgAdmins.map((member, i) =>
    new gcp.organizations.IAMMember(`org-admin-${i}`, {
        orgId: orgId,
        role: "roles/resourcemanager.organizationAdmin",
        member: member,
    })
);

// Folder-level access (example: platform team)
const platformFolderAdmin = new gcp.folder.IAMMember("platform-folder-admin", {
    folder: platformFolderId,
    role: "roles/resourcemanager.folderAdmin",
    member: "group:platform-team@yurikrupnik.com",
});

// Data team access
const dataFolderViewer = new gcp.folder.IAMMember("data-folder-viewer", {
    folder: dataFolderId,
    role: "roles/viewer",
    member: "group:data-team@yurikrupnik.com",
});
```

---

### 4. Networking (`04-networking/`)

**Purpose:** Shared networking infrastructure. Critical foundation that other projects depend on.

**Manages:**
- Shared VPC host project
- VPC networks and subnets
- Cloud NAT and Cloud Router
- DNS zones (Cloud DNS)
- Firewall rules
- VPN/Interconnect (if needed)
- Private Service Connect

**Dependencies:** Organization, IAM

**Run frequency:** When network topology changes

```
04-networking/
├── Pulumi.yaml
├── Pulumi.dev.yaml
├── Pulumi.staging.yaml
├── Pulumi.prod.yaml
├── src/
│   ├── index.ts
│   ├── vpc.ts
│   ├── subnets.ts
│   ├── nat.ts
│   ├── dns.ts
│   ├── firewall.ts
│   └── private-service-connect.ts
└── README.md
```

**Resources created:**
```typescript
// Shared VPC host project
const networkProject = new gcp.organizations.Project("network-host", {
    name: "network-host",
    projectId: "yurikrupnik-network-host",
    folderId: platformFolderId,
    billingAccount: billingAccountId,
});

// Enable Shared VPC
const sharedVpc = new gcp.compute.SharedVPCHostProject("shared-vpc-host", {
    project: networkProject.projectId,
});

// VPC
const vpc = new gcp.compute.Network("main-vpc", {
    name: "main-vpc",
    project: networkProject.projectId,
    autoCreateSubnetworks: false,
});

// Subnets per environment
const devSubnet = new gcp.compute.Subnetwork("dev-subnet", {
    name: "dev-subnet",
    project: networkProject.projectId,
    network: vpc.id,
    ipCidrRange: "10.0.0.0/20",
    region: "europe-west1",
    privateIpGoogleAccess: true,
    secondaryIpRanges: [
        { rangeName: "pods", ipCidrRange: "10.4.0.0/14" },
        { rangeName: "services", ipCidrRange: "10.8.0.0/20" },
    ],
});
```

---

### 5. Shared Services (`05-shared-services/`)

**Purpose:** Organization-wide shared services that multiple teams consume.

**Manages:**
- Artifact Registry (container images, packages)
- Secret Manager (org-wide secrets)
- KMS (encryption keys)
- Cloud Build (shared pipelines)
- Container Security (Binary Authorization)

**Dependencies:** Organization, IAM, Networking

**Run frequency:** When adding new shared capabilities

```
05-shared-services/
├── Pulumi.yaml
├── Pulumi.prod.yaml
├── src/
│   ├── index.ts
│   ├── artifact-registry.ts
│   ├── secret-manager.ts
│   ├── kms.ts
│   └── cloud-build.ts
└── README.md
```

**Resources created:**
```typescript
// Shared services project
const sharedProject = new gcp.organizations.Project("shared-services", {
    name: "shared-services",
    projectId: "yurikrupnik-shared",
    folderId: platformFolderId,
    billingAccount: billingAccountId,
});

// Artifact Registry for containers
const containerRegistry = new gcp.artifactregistry.Repository("containers", {
    project: sharedProject.projectId,
    location: "europe-west1",
    repositoryId: "containers",
    format: "DOCKER",
});

// KMS keyring for org-wide encryption
const keyring = new gcp.kms.KeyRing("org-keyring", {
    project: sharedProject.projectId,
    name: "org-keyring",
    location: "europe-west1",
});

const encryptionKey = new gcp.kms.CryptoKey("data-encryption", {
    name: "data-encryption",
    keyRing: keyring.id,
    rotationPeriod: "7776000s", // 90 days
});
```

---

### 6. Environments (`environments/`)

**Purpose:** Environment-specific resources that span multiple workloads.

**Structure:**
```
environments/
├── dev/
│   ├── Pulumi.yaml
│   ├── Pulumi.dev.yaml
│   └── src/
│       ├── index.ts
│       ├── project.ts        # Dev GCP project
│       ├── gke.ts            # Dev GKE cluster
│       ├── databases.ts      # Dev databases
│       └── monitoring.ts     # Dev monitoring
├── staging/
│   └── ... (same structure)
└── prod/
    └── ... (same structure)
```

**Resources per environment:**
```typescript
// Environment project
const envProject = new gcp.organizations.Project("dev-project", {
    name: "dev",
    projectId: "yurikrupnik-dev",
    folderId: devFolderId,
    billingAccount: billingAccountId,
});

// GKE cluster
const cluster = new gcp.container.Cluster("dev-cluster", {
    name: "dev-cluster",
    project: envProject.projectId,
    location: "europe-west1",
    initialNodeCount: 1,
    removeDefaultNodePool: true,
    workloadIdentityConfig: {
        workloadPool: `${envProject.projectId}.svc.id.goog`,
    },
});
```

---

### 7. Workloads (`workloads/`)

**Purpose:** Team or domain-specific infrastructure. Each team owns their workload project.

**Structure:**
```
workloads/
├── data/
│   ├── Pulumi.yaml
│   ├── Pulumi.dev.yaml
│   ├── Pulumi.prod.yaml
│   └── src/
│       ├── index.ts
│       ├── bigquery.ts
│       ├── dataflow.ts
│       ├── pubsub.ts
│       └── composer.ts
├── machine-learning/
│   └── src/
│       ├── index.ts
│       ├── vertex-ai.ts
│       ├── notebooks.ts
│       └── feature-store.ts
├── mlops/
│   └── src/
│       ├── index.ts
│       ├── pipelines.ts
│       ├── model-registry.ts
│       └── endpoints.ts
└── platform/
    └── src/
        ├── index.ts
        ├── cloud-run.ts
        ├── cloud-functions.ts
        └── api-gateway.ts
```

---

## Pulumi Stack Strategy

### Naming Convention

```
<project>/<environment>

Examples:
- organization/prod
- iam/prod
- networking/dev
- networking/prod
- workloads-data/dev
- workloads-data/prod
```

### Stack Configuration

Each stack has its own configuration file:

```yaml
# Pulumi.prod.yaml
config:
  gcp:project: yurikrupnik-prod
  gcp:region: europe-west1
  organization:orgId: "477132171939"
  organization:billingAccount: "XXXXXX-XXXXXX-XXXXXX"
```

---

## Deployment Order

Projects must be deployed in dependency order:

```
1. 01-bootstrap        (manual, one-time)
        ↓
2. 02-organization     (creates folders)
        ↓
3. 03-iam              (creates permissions)
        ↓
4. 04-networking       (creates VPC, subnets)
        ↓
5. 05-shared-services  (creates registries, KMS)
        ↓
6. environments/*      (creates env-specific infra)
        ↓
7. workloads/*         (creates workload resources)
```

### Automation Script

```bash
#!/bin/bash
# deploy-all.sh

set -e

ENVIRONMENT=${1:-dev}

echo "Deploying to $ENVIRONMENT..."

# Foundation (usually only prod stack)
cd 01-bootstrap && pulumi up -s prod --yes && cd ..
cd 02-organization && pulumi up -s prod --yes && cd ..
cd 03-iam && pulumi up -s prod --yes && cd ..

# Environment-specific
cd 04-networking && pulumi up -s $ENVIRONMENT --yes && cd ..
cd 05-shared-services && pulumi up -s prod --yes && cd ..
cd environments/$ENVIRONMENT && pulumi up -s $ENVIRONMENT --yes && cd ..

# Workloads
for workload in workloads/*/; do
    cd $workload && pulumi up -s $ENVIRONMENT --yes && cd ../..
done

echo "Deployment complete!"
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
# .github/workflows/infrastructure.yml
name: Infrastructure

on:
  push:
    branches: [main]
    paths:
      - 'infrastructure/**'
  pull_request:
    paths:
      - 'infrastructure/**'

jobs:
  preview:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        project: [02-organization, 03-iam, 04-networking]
    steps:
      - uses: actions/checkout@v4
      - uses: pulumi/actions@v5
        with:
          command: preview
          work-dir: infrastructure/${{ matrix.project }}
          stack-name: prod
        env:
          PULUMI_ACCESS_TOKEN: ${{ secrets.PULUMI_ACCESS_TOKEN }}
          GOOGLE_CREDENTIALS: ${{ secrets.GCP_CREDENTIALS }}

  deploy:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        project: [02-organization, 03-iam, 04-networking]
      max-parallel: 1  # Deploy sequentially
    steps:
      - uses: actions/checkout@v4
      - uses: pulumi/actions@v5
        with:
          command: up
          work-dir: infrastructure/${{ matrix.project }}
          stack-name: prod
        env:
          PULUMI_ACCESS_TOKEN: ${{ secrets.PULUMI_ACCESS_TOKEN }}
          GOOGLE_CREDENTIALS: ${{ secrets.GCP_CREDENTIALS }}
```

---

## State Management

### Recommended: Pulumi Cloud

Use Pulumi Cloud (app.pulumi.com) for:
- State storage
- Secrets encryption
- Deployment history
- RBAC for team access

### Alternative: Self-hosted GCS Backend

```bash
# Set backend to GCS
pulumi login gs://yurikrupnik-pulumi-state
```

---

## Security Considerations

### 1. Service Account Per Project

Each Pulumi project should have its own service account with minimal permissions:

| Project | Service Account | Roles |
|---------|-----------------|-------|
| organization | pulumi-org@... | `roles/resourcemanager.organizationAdmin` |
| iam | pulumi-iam@... | `roles/iam.organizationRoleAdmin` |
| networking | pulumi-network@... | `roles/compute.networkAdmin` |
| shared-services | pulumi-shared@... | `roles/artifactregistry.admin` |

### 2. Secrets Management

Never commit secrets to git. Use:

```bash
# Store secrets in Pulumi config (encrypted)
pulumi config set --secret gcp:credentials "$(cat sa-key.json)"

# Or use Workload Identity Federation (recommended)
pulumi config set gcp:project yurikrupnik-prod
# No credentials needed - uses OIDC
```

### 3. Policy as Code

Use Pulumi CrossGuard for policy enforcement:

```typescript
// policy/index.ts
import * as policy from "@pulumi/policy";

new policy.PolicyPack("gcp-security", {
    policies: [
        {
            name: "no-public-buckets",
            description: "Prohibit public GCS buckets",
            enforcementLevel: "mandatory",
            validateResource: policy.validateResourceOfType(
                gcp.storage.BucketIAMBinding,
                (binding, args, reportViolation) => {
                    if (binding.members?.includes("allUsers")) {
                        reportViolation("Bucket must not be public");
                    }
                }
            ),
        },
    ],
});
```

---

## Mapping to Your Current Structure

Based on your current GCP folder structure:

| GCP Folder | Pulumi Project | Notes |
|------------|----------------|-------|
| yurikrupnik.com (org) | 02-organization | Manages the org itself |
| data | workloads/data | BigQuery, Dataflow, etc. |
| dev | environments/dev | Dev environment resources |
| machine-learning | workloads/machine-learning | Vertex AI, notebooks |
| marketing | workloads/marketing | Marketing team resources |
| mlops | workloads/mlops | ML pipelines, model serving |
| platform | workloads/platform + 04-networking + 05-shared-services | Core platform |

---

## Getting Started

### Step 1: Initialize Bootstrap Project

```bash
mkdir -p infrastructure/01-bootstrap
cd infrastructure/01-bootstrap
pulumi new gcp-typescript --name bootstrap
```

### Step 2: Create State Bucket and Automation SA

Deploy bootstrap manually as yurik@yurikrupnik.com with Owner permissions.

### Step 3: Initialize Remaining Projects

```bash
for project in 02-organization 03-iam 04-networking 05-shared-services; do
    mkdir -p infrastructure/$project
    cd infrastructure/$project
    pulumi new gcp-typescript --name $project
    cd ../..
done
```

### Step 4: Configure Pulumi Backend

```bash
# Switch all projects to GCS backend
pulumi login gs://yurikrupnik-pulumi-state
```

### Step 5: Deploy in Order

Follow the deployment order above, starting with organization → iam → networking → etc.

---

## Summary

| Layer | Project | Changes How Often | Risk Level |
|-------|---------|-------------------|------------|
| Foundation | 01-bootstrap | Once | Critical |
| Foundation | 02-organization | Rarely | Critical |
| Foundation | 03-iam | Weekly | High |
| Foundation | 04-networking | Monthly | High |
| Shared | 05-shared-services | Monthly | Medium |
| Environment | environments/* | Weekly | Medium |
| Workload | workloads/* | Daily | Low |

This structure provides clear separation, minimizes blast radius, and enables teams to independently manage their workloads while maintaining organizational governance.

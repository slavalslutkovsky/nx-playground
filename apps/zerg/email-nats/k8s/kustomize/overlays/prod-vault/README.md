# Email Worker with Vault Integration

This overlay configures the email worker to use HashiCorp Vault for dynamic AWS SES credentials.

## Prerequisites

1. **Vault Server** - Running and accessible from GKE cluster
2. **Vault Agent Injector** - Installed in the cluster
3. **AWS Secrets Engine** - Configured in Vault
4. **Kubernetes Auth** - Configured in Vault

## Setup Steps

### 1. Install Vault Agent Injector (if not already installed)

```bash
helm repo add hashicorp https://helm.releases.hashicorp.com
helm install vault hashicorp/vault \
  --set "injector.enabled=true" \
  --set "server.enabled=false" \
  --set "injector.externalVaultAddr=https://vault.example.com"
```

### 2. Configure Vault AWS Secrets Engine

```bash
# Enable AWS secrets engine
vault secrets enable aws

# Configure root credentials (Vault uses these to generate dynamic creds)
vault write aws/config/root \
  access_key=$AWS_ACCESS_KEY_ID \
  secret_key=$AWS_SECRET_ACCESS_KEY \
  region=us-east-1

# Create role for SES access
vault write aws/roles/ses-sender \
  credential_type=iam_user \
  policy_arns=arn:aws:iam::aws:policy/AmazonSESFullAccess \
  default_ttl=1h \
  max_ttl=24h
```

### 3. Configure Vault Kubernetes Auth

```bash
# Enable Kubernetes auth
vault auth enable kubernetes

# Configure Kubernetes auth (from within the cluster or with proper kubeconfig)
vault write auth/kubernetes/config \
  kubernetes_host="https://kubernetes.default.svc:443"

# Create policy for email worker
vault policy write ses-sender - <<EOF
path "aws/creds/ses-sender" {
  capabilities = ["read"]
}
EOF

# Create role for email worker service account
vault write auth/kubernetes/role/email-worker \
  bound_service_account_names=email-worker \
  bound_service_account_namespaces=zerg \
  policies=ses-sender \
  ttl=1h
```

### 4. Deploy

```bash
# Apply the overlay
kubectl apply -k apps/zerg/email-nats/k8s/kustomize/overlays/prod-vault
```

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                         Pod                                  │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │   Vault Agent   │───▶│      /vault/secrets/aws-creds   │ │
│  │   (sidecar)     │    │  AWS_ACCESS_KEY_ID=...          │ │
│  └────────┬────────┘    │  AWS_SECRET_ACCESS_KEY=...      │ │
│           │             └─────────────────────────────────┘ │
│           │                           │                      │
│           │                           ▼                      │
│           │             ┌─────────────────────────────────┐ │
│           │             │     Email Worker Container      │ │
│           │             │   source /vault/secrets/...     │ │
│           │             │   → Uses AWS creds for SES      │ │
│           │             └─────────────────────────────────┘ │
└───────────┼─────────────────────────────────────────────────┘
            │
            ▼
     ┌──────────────┐
     │    Vault     │
     │  AWS Engine  │
     └──────────────┘
```

1. Pod starts with Vault Agent sidecar
2. Agent authenticates to Vault using K8s ServiceAccount token
3. Agent fetches AWS credentials from `aws/creds/ses-sender`
4. Credentials are written to `/vault/secrets/aws-creds`
5. Email worker sources the file to get credentials
6. Agent automatically refreshes before TTL expires

## Two Integration Options

### Option A: Vault Agent Sidecar (This Overlay)

- **Pros**: No code changes, works with any app
- **Cons**: Requires sourcing file, slight delay on refresh

### Option B: Direct Vault API (Code Integration)

Enable the `vault` feature and use `SesVaultProvider`:

```rust
// In Cargo.toml
email = { workspace = true, features = ["vault"] }

// In code
use email::provider::SesVaultProvider;

let provider = SesVaultProvider::from_env().await?;
provider.send(&email).await?;
```

Environment variables needed:
- `VAULT_ADDR` - Vault server URL
- `VAULT_ROLE` - Kubernetes auth role
- `VAULT_AWS_ROLE` - AWS secrets engine role
- `AWS_SES_REGION` - SES region
- `EMAIL_FROM_ADDRESS` - Sender email

## Troubleshooting

### Check Vault Agent logs
```bash
kubectl logs <pod-name> -c vault-agent
```

### Verify credentials are injected
```bash
kubectl exec <pod-name> -c email-nats-worker -- cat /vault/secrets/aws-creds
```

### Test Vault connectivity
```bash
kubectl exec <pod-name> -c vault-agent -- vault status
```

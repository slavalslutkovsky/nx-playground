# Environment Configuration Guide

This guide explains how to manage environment-specific configurations for the Terran API across different deployment environments.

## Environment Variables

### Core Configuration
- `PORT`: Server port (default: 8080)
- `RUST_LOG`: Logging level configuration
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_HOST`: Redis connection URL

### CORS & OAuth Configuration
- `CORS_ALLOWED_ORIGIN`: Frontend origin for CORS (e.g., http://localhost:3000)
- `REDIRECT_BASE_URL`: OAuth callback base URL (e.g., http://localhost:8080)
- `FRONTEND_URL`: Frontend application URL for post-auth redirects
- `OAUTH_AUTO_LINK_VERIFIED_EMAILS`: Auto-link OAuth accounts with verified emails (true/false)

### Authentication
- `JWT_SECRET`: Secret key for JWT token signing

### OAuth Providers
- `GOOGLE_CLIENT_ID`: Google OAuth client ID
- `GOOGLE_CLIENT_SECRET`: Google OAuth client secret
- `GITHUB_CLIENT_ID`: GitHub OAuth client ID
- `GITHUB_CLIENT_SECRET`: GitHub OAuth client secret

### Feature Flags (Flagsmith)
- `FLAGSMITH_API_URL`: Flagsmith API endpoint
- `FLAGSMITH_ENVIRONMENT_KEY`: Flagsmith environment key

## Understanding Environment Variables in Kubernetes

**Important:** These URLs are used by **your browser**, not by pods talking to each other!

See [DEPLOYMENT_SCENARIOS.md](./DEPLOYMENT_SCENARIOS.md) for detailed explanation of how these work in different setups (port-forward vs ingress vs production).

## Environment-Specific Configuration

### Local Development (Tilt)

For local Kubernetes development with Tilt:

**Ports:**
- API: http://localhost:5201 (port-forward 5201:8080)
- Web: http://localhost:5206 (port-forward 5206:80)

**Configuration (dev overlay):**
```yaml
CORS_ALLOWED_ORIGIN: "http://localhost:5206"
REDIRECT_BASE_URL: "http://localhost:5201"
FRONTEND_URL: "http://localhost:5206"
OAUTH_AUTO_LINK_VERIFIED_EMAILS: "true"
```

**Usage:**
```bash
tilt up
```

The dev overlay automatically configures all environment variables for local development.

### Production (GKE)

For production deployments on Google Kubernetes Engine:

**Configuration (prod overlay):**
```yaml
CORS_ALLOWED_ORIGIN: "https://your-production-domain.com"
REDIRECT_BASE_URL: "https://api.your-production-domain.com"
FRONTEND_URL: "https://your-production-domain.com"
OAUTH_AUTO_LINK_VERIFIED_EMAILS: "false"
```

## Security Best Practices

### Using Kubernetes Secrets

For production, sensitive values should be stored in Kubernetes Secrets:

1. **Create a secret from the template:**
   ```bash
   cd k8s/kustomize/overlays/prod
   cp secrets.example.yaml secrets.yaml
   # Edit secrets.yaml with your actual values
   ```

2. **Apply the secret:**
   ```bash
   kubectl apply -f secrets.yaml
   ```

3. **Update kustomization.yaml to use secrets:**
   ```yaml
   # Replace direct value:
   - name: JWT_SECRET
     value: "CHANGE-ME-use-k8s-secret"

   # With secret reference:
   - name: JWT_SECRET
     valueFrom:
       secretKeyRef:
         name: terran-api-secrets
         key: JWT_SECRET
   ```

### External Secrets Operator (Recommended)

For better secret management, use External Secrets Operator with cloud secret managers:

**GCP Secret Manager:**
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: terran-api-secrets
  namespace: terran
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: gcpsm-secret-store
    kind: SecretStore
  target:
    name: terran-api-secrets
  data:
    - secretKey: JWT_SECRET
      remoteRef:
        key: terran-api-jwt-secret
    - secretKey: GOOGLE_CLIENT_ID
      remoteRef:
        key: terran-api-google-client-id
    - secretKey: GOOGLE_CLIENT_SECRET
      remoteRef:
        key: terran-api-google-client-secret
```

## OAuth Setup

### Google OAuth
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create OAuth 2.0 credentials
3. Add authorized redirect URIs:
   - Local: `http://localhost:5201/oauth/google/callback`
   - Production: `https://api.your-domain.com/oauth/google/callback`

### GitHub OAuth
1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Create a new OAuth App
3. Set Authorization callback URL:
   - Local: `http://localhost:5201/oauth/github/callback`
   - Production: `https://api.your-domain.com/oauth/github/callback`

## Deploying Configuration Changes

### Dev (Tilt)
```bash
# Configuration updates are applied automatically with Tilt
tilt up
```

### Production (kubectl)
```bash
# Apply the production configuration
kubectl apply -k k8s/kustomize/overlays/prod

# Verify deployment
kubectl get pods -n terran
kubectl logs -n terran deployment/terran-api
```

### Production (ArgoCD)
Configuration changes are automatically synchronized when committed to the main branch.

## Troubleshooting

### Check environment variables in running pod:
```bash
kubectl exec -n terran deployment/terran-api -- env | grep -E "REDIRECT|FRONTEND|CORS|OAUTH"
```

### View logs:
```bash
kubectl logs -n terran deployment/terran-api -f
```

### Test OAuth flow:
```bash
# Local
curl http://localhost:5201/oauth/google

# Production
curl https://api.your-domain.com/oauth/google
```

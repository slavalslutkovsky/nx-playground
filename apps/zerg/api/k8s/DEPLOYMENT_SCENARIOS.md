# Deployment Scenarios and Environment Configuration

This document explains how environment variables like `FRONTEND_URL`, `REDIRECT_BASE_URL`, and `CORS_ALLOWED_ORIGIN` work in different deployment scenarios.

## The Challenge

These environment variables contain URLs that the **user's browser** needs to access, but they're configured **inside a Kubernetes pod**. This creates different requirements depending on how you access the services.

## Scenario 1: Tilt with Port-Forwards (Current Dev Setup)

### How It Works

```
┌─────────────────────┐
│  Developer Machine  │
│                     │
│  Browser            │
│  ↓                  │
│  localhost:5206 ────┼────► port-forward ────► terran-web pod:80
│  localhost:5201 ────┼────► port-forward ────► terran-api pod:8080
│                     │
└─────────────────────┘
```

### Configuration

```yaml
# dev/kustomization.yaml (current)
CORS_ALLOWED_ORIGIN: "http://localhost:5206"
REDIRECT_BASE_URL: "http://localhost:5201"
FRONTEND_URL: "http://localhost:5206"
```

### Why This Works

1. Port-forwards tunnel cluster services to your localhost
2. Browser loads web app from `http://localhost:5206`
3. Web app makes API calls to `http://localhost:8080` (or configured URL)
4. OAuth redirects go to `http://localhost:5201/oauth/...`
5. After OAuth, redirects to `http://localhost:5206/?token=...`

### Pros
- ✅ Simple - just `tilt up`
- ✅ No DNS configuration needed
- ✅ Works on any machine

### Cons
- ❌ Only one developer can use these ports at a time
- ❌ Port conflicts if running multiple projects
- ❌ Doesn't work for pod-to-pod communication
- ❌ Confusing (localhost inside K8s?)

### Usage

```bash
tilt up
# Access at http://localhost:5206
```

---

## Scenario 2: Ingress with Local DNS (Better Dev Setup)

### How It Works

```
┌─────────────────────┐
│  Developer Machine  │
│                     │
│  Browser            │
│  ↓                  │
│  terran.local ──────┼────► Ingress ────► terran-web service:80 ────► pod:80
│  api.terran.local ──┼────► Ingress ────► terran-api service:8080 ──► pod:8080
│                     │
└─────────────────────┘
```

### Configuration

```yaml
# dev/kustomization-ingress.yaml
CORS_ALLOWED_ORIGIN: "http://terran.local"
REDIRECT_BASE_URL: "http://api.terran.local"
FRONTEND_URL: "http://terran.local"
```

### Setup Required

**1. Install Ingress Controller:**
```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml
```

**2. Add to `/etc/hosts`:**
```bash
echo "127.0.0.1 terran.local api.terran.local" | sudo tee -a /etc/hosts
```

**3. Create Ingress:**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: terran
  namespace: terran
spec:
  ingressClassName: nginx
  rules:
  - host: terran.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: terran-web
            port:
              number: 80
  - host: api.terran.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: terran-api
            port:
              number: 8080
```

**4. Port-forward Ingress Controller:**
```bash
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller 80:80 443:443
```

### Pros
- ✅ Real domain names (easier to understand)
- ✅ Multiple developers can use different domains
- ✅ Closer to production setup
- ✅ Can add TLS easily

### Cons
- ❌ Requires ingress controller
- ❌ Need to edit /etc/hosts
- ❌ Still need port-forward for ingress controller

### Usage

```bash
# Apply ingress configuration
kubectl apply -f ingress.yaml

# Port-forward ingress controller
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller 80:80

# Access at http://terran.local
```

---

## Scenario 3: Production GKE with Load Balancer

### How It Works

```
┌──────────────┐
│   Internet   │
│      ↓       │
│  Public IP   │
│      ↓       │
│ Load Balancer│ ─────► Ingress ─────► Services ─────► Pods
│  (GCP LB)    │
└──────────────┘
```

### Configuration

```yaml
# prod/kustomization.yaml
CORS_ALLOWED_ORIGIN: "https://terran.yourdomain.com"
REDIRECT_BASE_URL: "https://api.terran.yourdomain.com"
FRONTEND_URL: "https://terran.yourdomain.com"
```

### Setup Required

**1. Configure DNS:**
```
terran.yourdomain.com     → Load Balancer IP
api.terran.yourdomain.com → Load Balancer IP
```

**2. Create Ingress with TLS:**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: terran
  namespace: terran
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - terran.yourdomain.com
    - api.terran.yourdomain.com
    secretName: terran-tls
  rules:
  - host: terran.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: terran-web
            port:
              number: 80
  - host: api.terran.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: terran-api
            port:
              number: 8080
```

**3. Configure OAuth Providers:**
- Google: Add `https://api.terran.yourdomain.com/oauth/google/callback` to authorized redirects
- GitHub: Add `https://api.terran.yourdomain.com/oauth/github/callback` to authorization callback URL

### Pros
- ✅ Production-ready
- ✅ TLS/HTTPS
- ✅ Real domain names
- ✅ Auto-scaling with GKE

### Cons
- ❌ Costs money (load balancer, etc.)
- ❌ Need real domain
- ❌ DNS propagation time

---

## Scenario 4: In-Cluster Communication

### Use Case
If the API needs to make HTTP calls to the frontend (unlikely but possible).

### Configuration

Use Kubernetes service DNS:

```yaml
FRONTEND_URL: "http://terran-web.terran.svc.cluster.local"
```

**Problem:** This won't work for OAuth redirects because the browser can't resolve cluster DNS!

**Solution:** Use different variables for different purposes:

```rust
// In your Rust code
pub fn get_frontend_url_external() -> String {
    env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

pub fn get_frontend_url_internal() -> String {
    env::var("FRONTEND_URL_INTERNAL")
        .unwrap_or_else(|_| "http://terran-web.terran.svc.cluster.local".to_string())
}
```

```yaml
# In K8s config
FRONTEND_URL: "http://localhost:5206"  # For browser redirects
FRONTEND_URL_INTERNAL: "http://terran-web.terran.svc.cluster.local"  # For pod-to-pod
```

---

## Comparison Table

| Scenario | Access From | URLs Used | Setup Complexity | Best For |
|----------|-------------|-----------|------------------|----------|
| **Port-Forward** | localhost:5201/5206 | localhost | Low | Quick dev, single developer |
| **Local Ingress** | terran.local | Custom domains | Medium | Team dev, realistic setup |
| **GKE Production** | yourdomain.com | Public domains | High | Production |
| **Pod-to-Pod** | N/A | cluster.local | Low | Internal services |

---

## Recommendations

### For Development (Tilt)

**Option A: Keep it simple (current)**
```yaml
# Use port-forwards and localhost
CORS_ALLOWED_ORIGIN: "http://localhost:5206"
REDIRECT_BASE_URL: "http://localhost:5201"
FRONTEND_URL: "http://localhost:5206"
```

**Option B: Use local ingress**
```yaml
# Set up ingress with local DNS
CORS_ALLOWED_ORIGIN: "http://terran.local"
REDIRECT_BASE_URL: "http://api.terran.local"
FRONTEND_URL: "http://terran.local"
```

### For Production

```yaml
CORS_ALLOWED_ORIGIN: "https://terran.yourdomain.com"
REDIRECT_BASE_URL: "https://api.terran.yourdomain.com"
FRONTEND_URL: "https://terran.yourdomain.com"
```

### Environment Variable Naming

Consider using clearer names:

```yaml
# External URLs (for browser)
BROWSER_FRONTEND_URL: "http://localhost:5206"
BROWSER_API_URL: "http://localhost:5201"

# Internal URLs (for pod-to-pod, if needed)
INTERNAL_FRONTEND_URL: "http://terran-web.terran.svc.cluster.local"
INTERNAL_API_URL: "http://terran-api.terran.svc.cluster.local:8080"
```

---

## Common Pitfalls

### 1. Using cluster DNS for browser URLs
```yaml
❌ FRONTEND_URL: "http://terran-web.terran.svc.cluster.local"
```
Browser can't resolve this!

### 2. Using localhost in production
```yaml
❌ REDIRECT_BASE_URL: "http://localhost:8080"  # in prod
```
Users' browsers aren't on the same machine as your cluster!

### 3. Wrong OAuth redirect URLs
Make sure your OAuth provider (Google, GitHub) has the correct callback URLs registered:
- Dev: `http://localhost:5201/oauth/google/callback`
- Prod: `https://api.yourdomain.com/oauth/google/callback`

---

## Testing Your Configuration

```bash
# Check environment variables in pod
kubectl exec -n terran deployment/terran-api -- env | grep -E "FRONTEND|REDIRECT|CORS"

# Test OAuth flow
# Dev:
curl -I http://localhost:5201/oauth/google

# Prod:
curl -I https://api.yourdomain.com/oauth/google

# Should see redirect to Google
```

---

## Switching Between Scenarios

To switch from port-forward to ingress:

```bash
# 1. Stop using current dev overlay
# Comment out in Tiltfile:
# k8s_yaml(kustomize('k8s/kustomize/overlays/dev'))

# 2. Use ingress overlay
k8s_yaml(kustomize('k8s/kustomize/overlays/dev-ingress'))

# 3. Remove port-forward from Tiltfile:
# k8s_resource("terran-api", port_forwards="5201:8080")
```

# Troubleshooting Guide

Common issues and their solutions for the nx-playground platform.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Service Issues](#service-issues)
- [Database Issues](#database-issues)
- [Agent Issues](#agent-issues)
- [Kubernetes Issues](#kubernetes-issues)
- [Performance Issues](#performance-issues)
- [Common Errors](#common-errors)

---

## Quick Diagnostics

### Health Check Commands

```bash
# Check all pods
kubectl get pods -n zerg -o wide

# Check service endpoints
kubectl get endpoints -n zerg

# Check recent events
kubectl get events -n zerg --sort-by='.lastTimestamp' | tail -20

# Check logs for errors
kubectl logs -l app=zerg-api -n zerg --tail=100 | grep -i error

# Check resource usage
kubectl top pods -n zerg
```

### Service Health URLs

| Service | Health URL | Ready URL |
|---------|------------|-----------|
| zerg-api | `GET /health` | `GET /ready` |
| zerg-tasks | gRPC health | gRPC health |
| agent-gateway | `GET /health` | `GET /ready` |
| email-nats | `GET /health` | `GET /ready` |

---

## Service Issues

### API Not Responding

**Symptoms:**
- 502/503 errors from ingress
- Connection timeouts

**Diagnosis:**

```bash
# Check pod status
kubectl get pods -l app=zerg-api -n zerg

# Check logs
kubectl logs -l app=zerg-api -n zerg --tail=200

# Check if service has endpoints
kubectl get endpoints zerg-api -n zerg

# Test from within cluster
kubectl run debug --image=curlimages/curl --rm -it -- \
  curl -v http://zerg-api.zerg.svc.cluster.local:8080/health
```

**Solutions:**

1. **Pod CrashLooping:**
   ```bash
   # Check crash reason
   kubectl describe pod <pod-name> -n zerg

   # Check previous logs
   kubectl logs <pod-name> -n zerg --previous
   ```

2. **Readiness probe failing:**
   ```bash
   # Check if database is accessible
   kubectl exec -it <pod-name> -n zerg -- \
     nc -zv postgres.dbs.svc.cluster.local 5432
   ```

3. **Missing secrets:**
   ```bash
   # Check if secrets exist
   kubectl get secrets -n zerg

   # Check ExternalSecret sync status
   kubectl get externalsecrets -n zerg
   ```

---

### gRPC Services Unavailable

**Symptoms:**
- "connection refused" errors
- "transport: Error while dialing"

**Diagnosis:**

```bash
# Check gRPC service
kubectl get pods -l app=zerg-tasks -n zerg

# Test gRPC connectivity
kubectl run grpcurl --image=fullstorydev/grpcurl --rm -it -- \
  -plaintext zerg-tasks.zerg.svc.cluster.local:50051 list

# Check network policies
kubectl get networkpolicies -n zerg
```

**Solutions:**

1. **Network policy blocking:**
   ```bash
   # Temporarily disable network policies for debugging
   kubectl delete networkpolicy default-deny-ingress -n zerg

   # Re-apply after testing
   kubectl apply -k k8s/hardening/base
   ```

2. **Service misconfigured:**
   ```bash
   # Check service ports
   kubectl get svc zerg-tasks -n zerg -o yaml

   # Ensure ports match deployment
   kubectl get deployment zerg-tasks -n zerg -o yaml | grep -A10 ports
   ```

---

## Database Issues

### PostgreSQL Connection Failures

**Symptoms:**
- "could not connect to server"
- "connection timed out"

**Diagnosis:**

```bash
# Check PostgreSQL pod
kubectl get pods -l app=postgres -n dbs

# Test connectivity
kubectl exec -it <api-pod> -n zerg -- \
  pg_isready -h postgres.dbs.svc.cluster.local -p 5432

# Check credentials
kubectl get secret postgres-credentials -n dbs -o jsonpath='{.data.password}' | base64 -d
```

**Solutions:**

1. **Connection pool exhausted:**
   ```bash
   # Check current connections
   kubectl exec -it <postgres-pod> -n dbs -- \
     psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"

   # Kill idle connections
   kubectl exec -it <postgres-pod> -n dbs -- \
     psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND query_start < now() - interval '10 minutes';"
   ```

2. **Wrong credentials:**
   ```bash
   # Verify ExternalSecret
   kubectl describe externalsecret database-secrets -n zerg

   # Force sync
   kubectl annotate externalsecret database-secrets -n zerg \
     force-sync=$(date +%s) --overwrite
   ```

---

### Redis Connection Issues

**Symptoms:**
- "NOAUTH Authentication required"
- "connection refused"

**Diagnosis:**

```bash
# Check Redis pod
kubectl get pods -l app=redis -n dbs

# Test connectivity
kubectl exec -it <api-pod> -n zerg -- \
  redis-cli -h redis.dbs.svc.cluster.local ping
```

**Solutions:**

```bash
# If password is wrong
kubectl get secret redis-credentials -n dbs -o jsonpath='{.data.password}' | base64 -d

# Restart Redis (caution: clears data)
kubectl rollout restart deployment/redis -n dbs
```

---

## Agent Issues

### Agent Gateway Errors

**Symptoms:**
- 502 when calling agents
- Agent health checks failing

**Diagnosis:**

```bash
# Check gateway logs
kubectl logs -l app=agent-gateway -n zerg --tail=200

# Check agent health
curl http://localhost:8080/ready  # If port-forwarding

# Check registered agents
curl http://localhost:8080/agents
```

**Solutions:**

1. **Agent not responding:**
   ```bash
   # Check individual agent
   kubectl logs -l app=rag-agent -n zerg

   # Restart agent
   kubectl rollout restart deployment/rag-agent -n zerg
   ```

2. **Rate limiting:**
   ```bash
   # Check rate limit status
   kubectl logs -l app=agent-gateway -n zerg | grep "rate limit"

   # Increase limit (dev only)
   kubectl set env deployment/agent-gateway RATE_LIMIT_MAX=500 -n zerg
   ```

---

### LLM API Errors

**Symptoms:**
- "rate_limit_exceeded"
- "invalid_api_key"
- High latency

**Diagnosis:**

```bash
# Check for rate limit errors
kubectl logs -l app=agent-gateway -n zerg | grep -i "rate.*limit\|429"

# Check API key validity
kubectl get secret llm-secrets -n zerg -o jsonpath='{.data.ANTHROPIC_API_KEY}' | base64 -d | head -c 10
```

**Solutions:**

1. **Rate limiting:**
   - Implement exponential backoff
   - Reduce request concurrency
   - Consider model caching

2. **Invalid API key:**
   ```bash
   # Update secret in Vault
   vault kv put secret/zerg/llm ANTHROPIC_API_KEY=sk-ant-...

   # Force sync
   kubectl annotate externalsecret llm-secrets -n zerg force-sync=$(date +%s) --overwrite

   # Restart pods to pick up new secret
   kubectl rollout restart deployment/agent-gateway -n zerg
   ```

---

## Kubernetes Issues

### Pod Stuck in Pending

**Diagnosis:**

```bash
kubectl describe pod <pod-name> -n zerg | grep -A10 Events
```

**Common causes:**

1. **Insufficient resources:**
   ```bash
   # Check node resources
   kubectl describe nodes | grep -A5 "Allocated resources"

   # Check resource quotas
   kubectl describe resourcequota -n zerg
   ```

2. **Node selector/affinity:**
   ```bash
   # Check if node matches selectors
   kubectl get nodes --show-labels
   ```

3. **PVC not bound:**
   ```bash
   kubectl get pvc -n zerg
   ```

---

### Pod CrashLoopBackOff

**Diagnosis:**

```bash
# Get crash reason
kubectl describe pod <pod-name> -n zerg | grep -A20 "State:"

# Check previous logs
kubectl logs <pod-name> -n zerg --previous
```

**Common causes:**

1. **Missing environment variables:**
   ```bash
   kubectl get pod <pod-name> -n zerg -o yaml | grep -A50 env:
   ```

2. **Failed health checks:**
   ```bash
   # Check probe configuration
   kubectl get deployment <name> -n zerg -o yaml | grep -A10 livenessProbe
   ```

---

### Network Policies Blocking Traffic

**Diagnosis:**

```bash
# List all network policies
kubectl get networkpolicies -n zerg -o wide

# Test connectivity from pod
kubectl exec -it <pod> -n zerg -- nc -zv <target> <port>
```

**Solution:**

```bash
# Temporarily allow all (for debugging)
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-all-temp
  namespace: zerg
spec:
  podSelector: {}
  ingress:
    - {}
  egress:
    - {}
  policyTypes:
    - Ingress
    - Egress
EOF

# Remove after debugging
kubectl delete networkpolicy allow-all-temp -n zerg
```

---

## Performance Issues

### High Latency

**Diagnosis:**

```bash
# Check Prometheus metrics
# Request latency p95
sum(rate(http_request_duration_seconds_bucket{le="1"}[5m])) / sum(rate(http_request_duration_seconds_count[5m]))

# Check pod resources
kubectl top pods -n zerg
```

**Solutions:**

1. **Scale horizontally:**
   ```bash
   kubectl scale deployment zerg-api --replicas=5 -n zerg
   ```

2. **Check database queries:**
   ```bash
   # Enable slow query log
   kubectl exec -it <postgres-pod> -n dbs -- \
     psql -U postgres -c "ALTER SYSTEM SET log_min_duration_statement = 1000;"
   ```

3. **Check gRPC connection pool:**
   ```rust
   // Increase pool size in config
   GrpcClientPool::new()
       .max_connections(20)
       .build()
   ```

---

### Memory Issues

**Diagnosis:**

```bash
# Check memory usage
kubectl top pods -n zerg

# Check for OOMKilled
kubectl get pods -n zerg -o jsonpath='{range .items[*]}{.metadata.name}{" "}{.status.containerStatuses[*].lastState.terminated.reason}{"\n"}{end}'
```

**Solutions:**

1. **Increase memory limits:**
   ```yaml
   resources:
     limits:
       memory: "1Gi"  # Increase from 512Mi
   ```

2. **Profile memory usage:**
   ```bash
   # Enable Rust memory profiling
   MALLOC_CONF=prof:true cargo run -p zerg_api
   ```

---

## Common Errors

### Error Reference

| Error | Cause | Solution |
|-------|-------|----------|
| `ECONNREFUSED` | Service not running | Check pod status, restart |
| `DEADLINE_EXCEEDED` | Request timeout | Increase timeout, check target service |
| `UNAVAILABLE` | Service unhealthy | Check health endpoint, logs |
| `RESOURCE_EXHAUSTED` | Rate limit or quota | Back off, increase limits |
| `PERMISSION_DENIED` | Auth failed | Check credentials, RBAC |
| `NOT_FOUND` | Wrong path/ID | Verify endpoint, resource exists |
| `ALREADY_EXISTS` | Duplicate resource | Use upsert or check before create |

---

## Getting Help

### Collecting Debug Information

```bash
# Generate diagnostic bundle
kubectl get all -n zerg -o yaml > zerg-resources.yaml
kubectl logs -l app=zerg-api -n zerg --all-containers --tail=1000 > api-logs.txt
kubectl describe pods -n zerg > pods-describe.txt
kubectl get events -n zerg --sort-by='.lastTimestamp' > events.txt

# Create tarball
tar -czvf debug-bundle.tar.gz zerg-resources.yaml api-logs.txt pods-describe.txt events.txt
```

### Useful Commands Cheatsheet

```bash
# Quick pod shell
kubectl exec -it $(kubectl get pods -l app=zerg-api -n zerg -o jsonpath='{.items[0].metadata.name}') -n zerg -- /bin/sh

# Follow all logs
kubectl logs -f -l app=zerg-api -n zerg --all-containers

# Watch pods
watch kubectl get pods -n zerg

# Port forward for local debugging
kubectl port-forward svc/zerg-api 8080:8080 -n zerg
```

---

## Related Documentation

- [Architecture Overview](./ARCHITECTURE.md)
- [Deployment Guide](./DEPLOYMENT.md)
- [Agents Guide](./AGENTS.md)

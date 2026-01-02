# Kubernetes Database Manifests

Kubernetes manifests for vector, graph, and other databases. Generated using Kompose from docker-compose.

## Generated Resources

### Vector Databases
| Database | Deployment | Service | PVC |
|----------|------------|---------|-----|
| **Qdrant** | qdrant-deployment.yaml | qdrant-service.yaml | qdrant-claim0-persistentvolumeclaim.yaml |
| **Milvus** | milvus-deployment.yaml | milvus-service.yaml | milvus-data-persistentvolumeclaim.yaml |
| Milvus etcd | milvus-etcd-deployment.yaml | - | milvus-etcd-persistentvolumeclaim.yaml |
| Milvus MinIO | milvus-minio-deployment.yaml | milvus-minio-service.yaml | milvus-minio-persistentvolumeclaim.yaml |

### Graph Databases
| Database | Deployment | Service | PVC |
|----------|------------|---------|-----|
| **Neo4j** | neo4j-deployment.yaml | neo4j-service.yaml | neo4j-data/logs-persistentvolumeclaim.yaml |
| **ArangoDB** | arangodb-deployment.yaml | arangodb-service.yaml | arangodb-data/apps-persistentvolumeclaim.yaml |

### Core Databases
| Database | Deployment | Service |
|----------|------------|---------|
| PostgreSQL | postgres-deployment.yaml | postgres-service.yaml |
| Redis | redis-deployment.yaml | redis-service.yaml |
| MongoDB | mongo-deployment.yaml | mongo-service.yaml |
| InfluxDB | influxdb2-deployment.yaml | influxdb2-service.yaml |

## Usage

### Deploy all databases
```bash
kubectl apply -k k8s/databases/
```

### Deploy specific databases only
```bash
# Vector databases only
kubectl apply -f k8s/databases/qdrant-deployment.yaml
kubectl apply -f k8s/databases/qdrant-service.yaml

# Graph databases only
kubectl apply -f k8s/databases/neo4j-deployment.yaml
kubectl apply -f k8s/databases/neo4j-service.yaml
```

### Check status
```bash
kubectl get pods -l app.kubernetes.io/part-of=nx-playground
kubectl get services
kubectl get pvc
```

### Port forwarding for local access
```bash
# Qdrant
kubectl port-forward svc/qdrant 6333:6333

# Neo4j
kubectl port-forward svc/neo4j 7474:7474 7687:7687

# ArangoDB
kubectl port-forward svc/arangodb 8529:8529

# Milvus
kubectl port-forward svc/milvus 19530:19530
```

## Regenerate from docker-compose

If you modify `manifests/dockers/compose.yaml`, regenerate the k8s manifests:

```bash
cd manifests/dockers
kompose convert -f compose.yaml -o ../../k8s/databases/
```

## Service Ports

| Service | Ports |
|---------|-------|
| Qdrant | 6333 (REST), 6334 (gRPC) |
| Neo4j | 7474 (HTTP), 7687 (Bolt) |
| ArangoDB | 8529 |
| Milvus | 19530 (gRPC), 9091 (metrics), 19121 (REST) |
| PostgreSQL | 5432 |
| Redis | 6379 |
| MongoDB | 27017 |
| InfluxDB | 8086 |

## Environment Variables for zerg-api

```yaml
env:
  # Qdrant
  - name: QDRANT_URL
    value: "http://qdrant:6333"

  # Neo4j
  - name: NEO4J_URI
    value: "bolt://neo4j:7687"
  - name: NEO4J_USER
    value: "neo4j"
  - name: NEO4J_PASSWORD
    valueFrom:
      secretKeyRef:
        name: neo4j-secret
        key: password

  # ArangoDB
  - name: ARANGO_URL
    value: "http://arangodb:8529"
  - name: ARANGO_USER
    value: "root"
  - name: ARANGO_PASSWORD
    valueFrom:
      secretKeyRef:
        name: arangodb-secret
        key: password
  - name: ARANGO_DATABASE
    value: "_system"

  # Milvus
  - name: MILVUS_URL
    value: "http://milvus:19121"
```

## Production Considerations

1. **Use proper secrets management** (External Secrets, Vault, etc.)
2. **Configure resource limits** in deployments
3. **Use StatefulSets** for databases requiring stable network IDs
4. **Configure proper storage classes** for PVCs
5. **Set up monitoring** (Prometheus, Grafana)
6. **Configure backup solutions** for persistent data

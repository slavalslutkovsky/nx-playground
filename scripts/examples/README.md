# Database Examples

Example scripts demonstrating how to use vector and graph databases for RAG applications.

## Prerequisites

Start the required databases using docker-compose:

```bash
cd manifests/dockers

# Start all databases
docker-compose up -d qdrant neo4j arangodb milvus milvus-etcd milvus-minio

# Or start individually
docker-compose up -d qdrant
docker-compose up -d neo4j
docker-compose up -d arangodb
docker-compose up -d milvus milvus-etcd milvus-minio
```

## Database Access

| Database | Web UI | API Port | Credentials |
|----------|--------|----------|-------------|
| Qdrant | http://localhost:6333/dashboard | 6333 (REST), 6334 (gRPC) | None |
| Neo4j | http://localhost:7474 | 7687 (Bolt) | neo4j / password123 |
| ArangoDB | http://localhost:8529 | 8529 | root / rootpassword |
| Milvus | - | 19530 (gRPC), 19121 (REST) | None |

## Example Scripts

### Qdrant (Vector Database)
```bash
./qdrant-examples.sh
```
Demonstrates:
- Collection management
- Vector upsert/search
- Filtered search
- Payload handling

### Neo4j (Graph Database)
```bash
./neo4j-examples.sh
```
Demonstrates:
- Node/relationship creation
- Cypher queries
- Graph traversal
- GraphRAG patterns (context retrieval)

### ArangoDB (Multi-Model)
```bash
./arangodb-examples.sh
```
Demonstrates:
- Document CRUD
- Edge collections
- Graph creation and traversal
- AQL queries
- Joins and aggregations

### Milvus (Vector Database)
```bash
./milvus-examples.sh
```
Demonstrates:
- Collection management
- Vector insert/search
- Index creation
- Scalar filtering

## Using with zerg-api

Set environment variables to enable database integrations:

```bash
# Qdrant
export QDRANT_URL=http://localhost:6333

# Neo4j
export NEO4J_URI=bolt://localhost:7687
export NEO4J_USER=neo4j
export NEO4J_PASSWORD=password123

# ArangoDB
export ARANGO_URL=http://localhost:8529
export ARANGO_USER=root
export ARANGO_PASSWORD=rootpassword
export ARANGO_DATABASE=_system

# Milvus
export MILVUS_URL=http://localhost:19121
```

Then run zerg-api:
```bash
cargo run -p zerg_api
```

API endpoints will be available at:
- `http://localhost:8080/api/qdrant/*`
- `http://localhost:8080/api/neo4j/*`
- `http://localhost:8080/api/arangodb/*`
- `http://localhost:8080/api/milvus/*`

## RAG Use Cases

### Vector Search (Qdrant/Milvus)
1. Embed documents using OpenAI/HuggingFace
2. Store embeddings with metadata
3. Query with embedded user question
4. Return top-k similar documents as context

### GraphRAG (Neo4j/ArangoDB)
1. Extract entities from documents
2. Build knowledge graph (nodes + relationships)
3. Query for entity context (traversal)
4. Combine with vector search for hybrid retrieval

### Hybrid RAG
```
User Query
    │
    ├──> Vector Search (semantic similarity)
    │         │
    │         └──> Top-k documents
    │
    └──> Graph Traversal (entity relationships)
              │
              └──> Related context
    │
    └──> Merge & Rerank
              │
              └──> Final context for LLM
```

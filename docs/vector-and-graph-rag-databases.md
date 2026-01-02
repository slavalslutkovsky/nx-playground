# Vector & Graph RAG Databases Comparison

Comparing databases for Retrieval Augmented Generation (RAG) workloads, including vector stores and graph databases.

## Vector Databases Overview

| Database | Type | Language | Filtering | Hybrid Search | License |
|----------|------|----------|-----------|---------------|---------|
| pgvector | PostgreSQL extension | C | SQL | Yes (with pg_bm25) | PostgreSQL |
| Pinecone | Managed service | - | Metadata | Yes | Commercial |
| Weaviate | Purpose-built | Go | GraphQL | Yes | BSD-3 |
| Milvus | Purpose-built | Go/C++ | Expressions | Yes | Apache 2.0 |
| Qdrant | Purpose-built | Rust | JSON | Yes | Apache 2.0 |
| Chroma | Embedded/Server | Python | Metadata | No | Apache 2.0 |
| LanceDB | Embedded | Rust | SQL | Yes | Apache 2.0 |

---

## pgvector

### Overview
PostgreSQL extension adding vector similarity search capabilities.

### Strengths
- **PostgreSQL ecosystem**: Use existing tools, backups, replication
- **ACID transactions**: Vectors with relational data in same transaction
- **SQL filtering**: Complex WHERE clauses with vector search
- **JOINs**: Combine vector results with relational data
- **No new infrastructure**: Add to existing PostgreSQL
- **Mature operations**: Leverage PostgreSQL expertise

### Weaknesses
- Performance lags behind purpose-built solutions at scale
- Limited index types (IVFFlat, HNSW)
- Memory-intensive for large datasets
- No built-in sharding for vectors

### Index Types
```sql
-- HNSW (recommended for most cases)
CREATE INDEX ON items USING hnsw (embedding vector_cosine_ops);

-- IVFFlat (faster builds, lower recall)
CREATE INDEX ON items USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);
```

### Example Query
```sql
-- Semantic search with metadata filtering
SELECT id, content, embedding <=> $1 AS distance
FROM documents
WHERE category = 'technical'
  AND created_at > NOW() - INTERVAL '30 days'
ORDER BY embedding <=> $1
LIMIT 10;
```

### Best For
- Teams already using PostgreSQL
- Applications needing ACID + vectors
- Moderate scale (<10M vectors)
- Complex filtering requirements

---

## Pinecone

### Strengths
- Fully managed, zero operations
- Excellent query performance
- Scales to billions of vectors
- Built-in metadata filtering
- Hybrid search (sparse + dense)
- Namespaces for multi-tenancy

### Weaknesses
- Vendor lock-in
- Cost at scale
- No self-hosted option
- Limited query flexibility vs SQL

### Example
```python
index.query(
    vector=[0.1, 0.2, ...],
    filter={"category": {"$eq": "technical"}},
    top_k=10,
    include_metadata=True
)
```

### Best For
- Startups wanting zero ops
- Rapid prototyping
- When budget allows managed services

---

## Weaviate

### Strengths
- GraphQL API
- Built-in vectorization modules (OpenAI, Cohere, HuggingFace)
- Hybrid search (BM25 + vector)
- Multi-modal (text, images)
- Schema-based with references
- Kubernetes-native

### Weaknesses
- GraphQL learning curve
- Resource-intensive
- Complex for simple use cases

### Example
```graphql
{
  Get {
    Document(
      nearText: { concepts: ["machine learning"] }
      where: { path: ["category"], operator: Equal, valueText: "technical" }
    ) {
      content
      _additional { certainty }
    }
  }
}
```

### Best For
- Multi-modal RAG
- When built-in vectorization is valuable
- GraphQL-native teams

---

## Milvus / Zilliz

### Strengths
- Scales to billions of vectors
- Multiple index types (IVF, HNSW, DiskANN, GPU)
- Attribute filtering
- Distributed architecture
- GPU acceleration
- Zilliz Cloud (managed)

### Weaknesses
- Operational complexity (self-hosted)
- Resource requirements
- Steeper learning curve

### Example
```python
collection.search(
    data=[[0.1, 0.2, ...]],
    anns_field="embedding",
    param={"metric_type": "COSINE", "params": {"nprobe": 10}},
    limit=10,
    expr="category == 'technical'"
)
```

### Best For
- Large-scale production deployments
- When GPU acceleration needed
- Enterprise requirements

---

## Qdrant

### Strengths
- Written in Rust (performance + safety)
- Rich filtering (nested JSON)
- Payload indexing
- Quantization support
- Hybrid search
- Simple deployment
- Good documentation

### Weaknesses
- Smaller community than Milvus
- Fewer index options
- Newer project

### Example
```python
client.search(
    collection_name="documents",
    query_vector=[0.1, 0.2, ...],
    query_filter=Filter(
        must=[FieldCondition(key="category", match=MatchValue(value="technical"))]
    ),
    limit=10
)
```

### Best For
- Production workloads needing reliability
- Complex filtering requirements
- Teams valuing simplicity

---

## Chroma

### Strengths
- Extremely simple API
- Embedded mode (no server needed)
- Python-native
- Great for prototyping
- LangChain/LlamaIndex integration

### Weaknesses
- Limited scalability
- No hybrid search
- Basic filtering
- Not for production scale

### Example
```python
collection.query(
    query_embeddings=[[0.1, 0.2, ...]],
    n_results=10,
    where={"category": "technical"}
)
```

### Best For
- Prototyping and development
- Small datasets (<100K vectors)
- Embedded applications

---

## LanceDB

### Strengths
- Embedded (serverless)
- Built on Lance format (columnar)
- SQL filtering
- Versioning built-in
- Zero-copy integration with ML tools
- Disk-based (low memory)

### Weaknesses
- Newer project
- Smaller ecosystem
- Limited distributed support

### Best For
- Embedded/edge deployments
- Data versioning needs
- Cost-sensitive applications

---

## Graph RAG Databases

Graph databases excel at representing relationships, enabling GraphRAG - combining knowledge graphs with retrieval.

| Database | Type | Query Language | Vector Support | License |
|----------|------|----------------|----------------|---------|
| Neo4j | Native Graph | Cypher | Yes (native) | GPL / Commercial |
| Amazon Neptune | Managed Graph | Gremlin/SPARQL/openCypher | Yes | Commercial |
| ArangoDB | Multi-model | AQL | Yes | Apache 2.0 |
| NebulaGraph | Native Graph | nGQL | Via plugin | Apache 2.0 |
| Dgraph | Native Graph | DQL/GraphQL | No (external) | Apache 2.0 |
| FalkorDB | Redis-based | Cypher | Yes | Source Available |

---

## Neo4j

### Overview
Leading graph database with native vector search support.

### Strengths
- **Native vector index**: HNSW built into database
- **Cypher**: Expressive graph query language
- **Graph + Vector**: Combine relationship traversal with similarity
- **Mature ecosystem**: Tools, drivers, community
- **GraphRAG patterns**: Well-documented approaches
- **AuraDB**: Managed cloud offering

### Weaknesses
- Commercial license for enterprise features
- Memory-intensive for large graphs
- Vector support is newer

### Vector + Graph Example
```cypher
// Create vector index
CREATE VECTOR INDEX document_embeddings
FOR (d:Document)
ON d.embedding
OPTIONS {indexConfig: {
  `vector.dimensions`: 1536,
  `vector.similarity_function`: 'cosine'
}};

// GraphRAG query: find similar docs, then traverse relationships
CALL db.index.vector.queryNodes('document_embeddings', 5, $queryVector)
YIELD node AS doc, score
MATCH (doc)-[:REFERENCES]->(cited:Document)
MATCH (doc)-[:AUTHORED_BY]->(author:Person)
RETURN doc.title, score, collect(cited.title) AS references, author.name
ORDER BY score DESC;
```

### GraphRAG Patterns
```cypher
// Entity extraction + relationship retrieval
MATCH (e:Entity {name: $extracted_entity})
MATCH path = (e)-[*1..2]-(related)
WITH related, length(path) AS distance
RETURN related.name, related.description, distance
ORDER BY distance;

// Community-based retrieval
MATCH (d:Document)-[:BELONGS_TO]->(c:Community)
WHERE c.topic = $query_topic
CALL db.index.vector.queryNodes('document_embeddings', 10, $queryVector)
YIELD node, score
WHERE node IN collect(d)
RETURN node.content, score;
```

### Best For
- Complex relationship queries
- Knowledge graph construction
- Entity-centric RAG
- When context requires graph traversal

---

## Amazon Neptune

### Strengths
- Fully managed
- Multiple query languages
- Neptune Analytics for vectors
- Graph + ML integration
- Enterprise-grade

### Weaknesses
- AWS lock-in
- Cost
- Less flexible than Neo4j for some patterns

### Best For
- AWS-native architectures
- Enterprise compliance needs

---

## ArangoDB

### Strengths
- **Multi-model**: Graph + Document + Key-Value + Search
- Native ArangoSearch (full-text + vector)
- Single query language (AQL)
- Flexible schema
- Good horizontal scaling

### Weaknesses
- Jack of all trades concerns
- Smaller graph-specific community
- Vector support less mature than specialists

### Example
```aql
// Combine vector search with graph traversal
LET similar = (
  FOR doc IN documents
    SEARCH ANALYZER(doc.embedding IN RANGE(@queryVector, 0.8, 1.0), "vector")
    LIMIT 10
    RETURN doc
)
FOR doc IN similar
  FOR v, e, p IN 1..2 OUTBOUND doc GRAPH 'knowledge_graph'
    RETURN { document: doc, related: v, path: p }
```

### Best For
- Multi-model requirements
- When you need graph + document + search
- Avoiding multiple databases

---

## FalkorDB (formerly RedisGraph)

### Strengths
- Redis-based (fast, familiar)
- Cypher support
- Vector similarity built-in
- Low latency
- Simple deployment

### Weaknesses
- Source-available license
- Redis dependency
- Smaller community

### Best For
- Low-latency requirements
- Redis-native architectures
- Caching layer with graph capabilities

---

## GraphRAG Architecture Patterns

### 1. Entity-Centric RAG
```
Query → Extract Entities → Graph Lookup → Retrieve Related Context → LLM
```

### 2. Hierarchical RAG (Microsoft GraphRAG)
```
Documents → Entity Extraction → Community Detection → Summaries at Multiple Levels
Query → Route to Appropriate Level → Retrieve Summaries → LLM
```

### 3. Hybrid Vector + Graph
```
Query → Vector Search (semantic) → Graph Expansion (relationships) → Rerank → LLM
```

### 4. Knowledge Graph QA
```
Query → LLM extracts Cypher/SPARQL → Execute on Graph → Return structured answer
```

---

## Comparison Matrix

### Vector Database Selection

| Requirement | Recommended |
|-------------|-------------|
| Already using PostgreSQL | pgvector |
| Zero operations | Pinecone |
| Multi-modal | Weaviate |
| Billion-scale | Milvus/Zilliz |
| Production + simplicity | Qdrant |
| Prototyping | Chroma |
| Embedded/Edge | LanceDB |

### Graph RAG Selection

| Requirement | Recommended |
|-------------|-------------|
| Best graph + vector combo | Neo4j |
| AWS native | Neptune |
| Multi-model flexibility | ArangoDB |
| Low latency | FalkorDB |
| Open source priority | NebulaGraph, Dgraph |

### Combined Selection

| Scenario | Recommendation |
|----------|----------------|
| Simple RAG, PostgreSQL stack | pgvector |
| Complex relationships matter | Neo4j with vectors |
| Multi-model (docs + graph + search) | ArangoDB |
| Maximum vector performance | Qdrant or Milvus + separate graph DB |
| Startup/MVP | Chroma → Pinecone |
| Enterprise knowledge graph | Neo4j AuraDB or Neptune |

---

## Performance Considerations

### Vector Search Latency (approximate, 1M vectors)

| Database | p50 Latency | p99 Latency |
|----------|-------------|-------------|
| Pinecone | 5-10ms | 20-50ms |
| Qdrant | 5-15ms | 30-80ms |
| Milvus | 5-15ms | 30-100ms |
| pgvector | 10-50ms | 100-300ms |
| Weaviate | 10-30ms | 50-150ms |

*Note: Highly dependent on hardware, index configuration, and filtering complexity.*

### Graph Traversal + Vector

Combining graph traversal with vector search adds latency but provides richer context:
- Simple traversal (1-2 hops): +5-20ms
- Complex patterns: +50-200ms
- Worth it when relationships are essential for answer quality

---

## Migration Paths

### pgvector → Purpose-built
1. Export embeddings to Parquet/CSV
2. Maintain PostgreSQL for metadata
3. Query vector DB, JOIN results in application

### Single Vector DB → Hybrid (Vector + Graph)
1. Extract entities from documents
2. Build knowledge graph in Neo4j/ArangoDB
3. Keep vectors in specialized DB or migrate to Neo4j vectors
4. Implement hybrid retrieval in application layer

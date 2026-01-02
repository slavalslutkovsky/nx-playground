# PostgreSQL vs Cassandra

A comparison of two fundamentally different database paradigms.

## Overview

| Aspect | PostgreSQL | Cassandra |
|--------|------------|-----------|
| Type | Relational (SQL) | Wide-column NoSQL |
| Architecture | Single-master | Masterless, peer-to-peer |
| Consistency | Strong (ACID) | Tunable (eventual by default) |
| Query Language | SQL | CQL (Cassandra Query Language) |
| Schema | Rigid, predefined | Flexible, schema-per-table |

## When to Use PostgreSQL

### Strengths
- **Complex queries**: JOINs, subqueries, window functions, CTEs
- **ACID transactions**: Multi-row, multi-table transactions
- **Data integrity**: Foreign keys, constraints, triggers
- **Rich data types**: JSON/JSONB, arrays, geometric types, full-text search
- **Extensions**: PostGIS, pg_vector, TimescaleDB, Citus

### Best For
- Traditional OLTP workloads
- Applications requiring complex relationships
- Financial systems needing strict consistency
- Reporting and analytics
- Geospatial applications
- Small to medium scale (vertical scaling)

### Limitations
- Horizontal scaling is complex (requires Citus or similar)
- Single point of failure without proper HA setup
- Write-heavy workloads can bottleneck

## When to Use Cassandra

### Strengths
- **Linear scalability**: Add nodes to increase capacity
- **High availability**: No single point of failure
- **Write performance**: Optimized for write-heavy workloads
- **Geographic distribution**: Multi-datacenter replication built-in
- **Time-series friendly**: Efficient for time-ordered data

### Best For
- Write-heavy applications (IoT, logging, metrics)
- Applications requiring 99.99%+ uptime
- Global applications needing multi-region deployment
- Time-series data at scale
- Simple query patterns with known access paths

### Limitations
- No JOINs (denormalization required)
- Limited ad-hoc query capability
- Eventual consistency can complicate application logic
- Requires careful data modeling upfront
- Operational complexity

## Data Modeling Comparison

### PostgreSQL (Normalized)
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE,
    name VARCHAR(100)
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    total DECIMAL(10,2),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Query with JOIN
SELECT u.name, o.total
FROM users u
JOIN orders o ON u.id = o.user_id;
```

### Cassandra (Denormalized)
```cql
CREATE TABLE orders_by_user (
    user_id UUID,
    order_id TIMEUUID,
    user_name TEXT,
    user_email TEXT,
    total DECIMAL,
    PRIMARY KEY (user_id, order_id)
) WITH CLUSTERING ORDER BY (order_id DESC);

-- Query (no JOINs)
SELECT * FROM orders_by_user WHERE user_id = ?;
```

## Consistency Models

### PostgreSQL
- Default: Serializable or Read Committed
- All reads see latest committed data
- Transactions are atomic across tables

### Cassandra
Tunable per-query:
| Level | Description |
|-------|-------------|
| ONE | Single replica responds |
| QUORUM | Majority of replicas respond |
| ALL | All replicas respond |
| LOCAL_QUORUM | Majority in local datacenter |

## Scaling Comparison

| Aspect | PostgreSQL | Cassandra |
|--------|------------|-----------|
| Vertical | Excellent | Good |
| Horizontal reads | Read replicas | Native |
| Horizontal writes | Complex (sharding) | Native |
| Max practical size | ~10TB per node | Petabytes |
| Rebalancing | Manual | Automatic |

## Operational Considerations

### PostgreSQL
- Mature tooling (pg_dump, pg_basebackup)
- Streaming replication for HA
- Requires careful vacuum tuning
- Many managed options (RDS, Cloud SQL, Aurora)

### Cassandra
- Requires JVM tuning
- Repair operations needed regularly
- Compaction can cause latency spikes
- Managed options: DataStax Astra, Amazon Keyspaces

## Cost Comparison (Rough)

| Scale | PostgreSQL | Cassandra |
|-------|------------|-----------|
| Small (<100GB) | Lower | Higher (min 3 nodes) |
| Medium (100GB-1TB) | Similar | Similar |
| Large (>1TB) | Higher (complex sharding) | Lower (linear scaling) |

## Decision Matrix

Choose **PostgreSQL** if:
- You need complex queries and JOINs
- ACID transactions are required
- Data relationships are complex
- Team has SQL expertise
- Scale is moderate (<10TB)

Choose **Cassandra** if:
- Write throughput is critical
- You need 99.99%+ availability
- Data model is simple and known upfront
- Scale will exceed single-node capacity
- Multi-region deployment is required

## Hybrid Approaches

Many architectures use both:
- PostgreSQL for transactional data and complex queries
- Cassandra for high-volume event streams and time-series

Consider also:
- **ScyllaDB**: Cassandra-compatible, better performance
- **CockroachDB**: PostgreSQL-compatible, distributed
- **YugabyteDB**: PostgreSQL-compatible, distributed

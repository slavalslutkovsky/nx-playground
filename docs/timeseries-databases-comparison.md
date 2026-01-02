# Time-Series Databases Comparison

Comparing InfluxDB with other popular time-series database solutions.

## Overview

| Database | Language | Storage Engine | Query Language | License |
|----------|----------|----------------|----------------|---------|
| InfluxDB | Go | TSM (custom) | InfluxQL / Flux | MIT (OSS) / Commercial |
| TimescaleDB | C | PostgreSQL | SQL | Apache 2.0 / Commercial |
| Prometheus | Go | Custom TSDB | PromQL | Apache 2.0 |
| QuestDB | Java/C++ | Custom | SQL | Apache 2.0 |
| ClickHouse | C++ | MergeTree | SQL | Apache 2.0 |
| VictoriaMetrics | Go | Custom | MetricsQL | Apache 2.0 |
| TDengine | C | Custom | SQL | AGPL / Commercial |

## InfluxDB

### Strengths
- Purpose-built for time-series from the ground up
- Excellent write performance
- Built-in retention policies and downsampling
- Strong ecosystem (Telegraf, Chronograf, Kapacitor - TICK stack)
- Good compression ratios
- InfluxDB 3.0 uses Apache Arrow and DataFusion

### Weaknesses
- Flux language has steep learning curve
- Clustering only in commercial version (pre-3.0)
- Memory-intensive for high cardinality
- Breaking changes between major versions

### Best For
- Metrics and monitoring
- IoT sensor data
- DevOps observability
- Real-time analytics

### Example Query (Flux)
```flux
from(bucket: "metrics")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "cpu")
  |> filter(fn: (r) => r._field == "usage_percent")
  |> aggregateWindow(every: 5m, fn: mean)
```

## TimescaleDB

### Strengths
- Full PostgreSQL compatibility (JOINs, extensions, tools)
- No new query language to learn
- Automatic partitioning (hypertables)
- Continuous aggregates
- Compression (up to 95%)
- Mature ecosystem of PostgreSQL tools

### Weaknesses
- Single-node in community edition (multi-node is commercial)
- Higher resource usage than purpose-built TSDBs
- Complex queries can be slower than specialized solutions

### Best For
- Teams with PostgreSQL expertise
- Applications needing relational + time-series
- Complex analytical queries
- When you need JOINs with time-series data

### Example Query (SQL)
```sql
SELECT time_bucket('5 minutes', time) AS bucket,
       avg(cpu_usage) as avg_cpu
FROM metrics
WHERE time > NOW() - INTERVAL '1 hour'
GROUP BY bucket
ORDER BY bucket;
```

## Prometheus

### Strengths
- De facto standard for Kubernetes monitoring
- Pull-based model simplifies service discovery
- Powerful alerting (Alertmanager)
- Large ecosystem of exporters
- PromQL is expressive for metrics

### Weaknesses
- Not designed for long-term storage
- No native clustering
- Pull model doesn't suit all use cases
- Limited to metrics (not events/logs)

### Best For
- Kubernetes monitoring
- Microservices observability
- Alert-driven workflows
- Short to medium retention (weeks)

### Example Query (PromQL)
```promql
rate(http_requests_total{job="api-server"}[5m])
```

## QuestDB

### Strengths
- Extremely fast ingestion (millions of rows/sec)
- SQL support with time-series extensions
- Low latency queries
- Efficient storage
- Simple deployment (single binary)

### Weaknesses
- Smaller community than alternatives
- Fewer integrations
- Limited clustering support
- Newer, less battle-tested

### Best For
- High-frequency trading data
- Real-time analytics requiring low latency
- When SQL is preferred over custom query languages

### Example Query (SQL)
```sql
SELECT timestamp, avg(price)
FROM trades
WHERE timestamp > dateadd('h', -1, now())
SAMPLE BY 1m;
```

## ClickHouse

### Strengths
- Exceptional analytical query performance
- Excellent compression
- SQL support
- Scales horizontally
- Great for large-scale analytics

### Weaknesses
- Not strictly a TSDB (general OLAP)
- More complex to operate
- Updates/deletes are expensive
- Requires more tuning

### Best For
- Large-scale analytics
- When you need OLAP + time-series
- Log analytics
- Business intelligence on time-series

### Example Query (SQL)
```sql
SELECT
    toStartOfFiveMinute(timestamp) AS ts,
    avg(value) AS avg_value
FROM metrics
WHERE timestamp >= now() - INTERVAL 1 HOUR
GROUP BY ts
ORDER BY ts;
```

## VictoriaMetrics

### Strengths
- Drop-in Prometheus replacement
- Better compression than Prometheus
- Lower resource usage
- Native clustering in open-source version
- Long-term storage for Prometheus
- MetricsQL (PromQL superset)

### Weaknesses
- Focused on metrics (not general time-series)
- Smaller ecosystem than Prometheus
- Less mature than some alternatives

### Best For
- Prometheus long-term storage
- High-cardinality metrics
- Cost-efficient metrics storage
- Prometheus at scale

## TDengine

### Strengths
- Extremely high write throughput
- Built-in caching, streaming, and subscriptions
- SQL support
- Clustering in open-source
- Low resource footprint

### Weaknesses
- AGPL license may be restrictive
- Smaller Western community
- Fewer integrations

### Best For
- IoT at massive scale
- Edge computing scenarios
- When write performance is critical

## Performance Comparison (Approximate)

| Database | Write (rows/sec) | Compression | Query Speed |
|----------|------------------|-------------|-------------|
| InfluxDB | 500K+ | High | Fast |
| TimescaleDB | 200K+ | Very High | Fast (SQL) |
| Prometheus | 100K+ | Medium | Fast |
| QuestDB | 2M+ | High | Very Fast |
| ClickHouse | 1M+ | Very High | Very Fast |
| VictoriaMetrics | 500K+ | Very High | Fast |
| TDengine | 2M+ | High | Fast |

*Note: Actual performance varies significantly based on hardware, schema, and query patterns.*

## Decision Matrix

| Requirement | Recommended |
|-------------|-------------|
| Kubernetes monitoring | Prometheus + VictoriaMetrics |
| PostgreSQL ecosystem | TimescaleDB |
| Maximum write throughput | QuestDB, TDengine |
| General metrics/monitoring | InfluxDB, VictoriaMetrics |
| Large-scale analytics | ClickHouse |
| Prometheus long-term storage | VictoriaMetrics, Thanos |
| IoT at scale | TDengine, InfluxDB |
| Simple SQL queries | TimescaleDB, QuestDB |

## Architecture Considerations

### Single Node
- QuestDB, InfluxDB OSS, Prometheus

### Native Clustering (Open Source)
- VictoriaMetrics, ClickHouse, TDengine

### Clustering (Commercial)
- InfluxDB Enterprise, TimescaleDB (multi-node)

### Cloud-Managed Options
- InfluxDB Cloud
- Timescale Cloud
- Grafana Cloud (Prometheus/Mimir)
- ClickHouse Cloud
- Amazon Timestream
- Azure Data Explorer
- Google Cloud Bigtable (with adaptations)

## Summary Recommendations

1. **Starting out / General purpose**: InfluxDB or TimescaleDB
2. **Kubernetes-native**: Prometheus + VictoriaMetrics for long-term
3. **Need SQL**: TimescaleDB or QuestDB
4. **Maximum performance**: QuestDB or ClickHouse
5. **Cost-conscious at scale**: VictoriaMetrics
6. **IoT/Edge**: TDengine or InfluxDB

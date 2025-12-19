# Observability

Prometheus metrics and observability utilities for the cloud cost optimization platform.

## Features

- **Prometheus Metrics** - Counter, gauge, and histogram metrics
- **HTTP Request Metrics** - Automatic request/response tracking via middleware
- **Pricing Metrics** - Domain-specific metrics for pricing operations
- **Resource Metrics** - Cloud resource inventory and sync metrics

## Quick Start

### Initialize Metrics

```rust
use observability::init_metrics;

fn main() {
    // Initialize Prometheus recorder (call once at startup)
    init_metrics();
}
```

### Add Metrics Endpoint

```rust
use axum::{Router, routing::get};
use observability::metrics_handler;

let app = Router::new()
    .route("/metrics", get(metrics_handler));
```

### Add Request Metrics Middleware

```rust
use axum::{Router, middleware};
use observability::middleware::metrics_middleware;

let app = Router::new()
    .route("/api/prices", get(list_prices))
    .layer(middleware::from_fn(metrics_middleware));
```

## Available Metrics

### HTTP Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `http_requests_total` | Counter | method, path, status, status_class | Total HTTP requests |
| `http_request_duration_seconds` | Histogram | method, path | Request duration |
| `http_requests_errors_total` | Counter | method, path, status | 4xx/5xx errors |

### Pricing Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `pricing_operations_total` | Counter | operation, provider, resource_type | Pricing operations |
| `pricing_operation_duration_seconds` | Histogram | operation, provider | Operation duration |
| `pricing_entries_total` | Gauge | provider, resource_type | Total pricing entries |
| `pricing_comparisons_total` | Counter | - | Price comparison requests |
| `price_collection_jobs_total` | Counter | provider, job_type, status | Collection jobs |
| `price_collection_duration_seconds` | Histogram | provider, job_type | Collection duration |
| `prices_collected_last_run` | Gauge | provider | Prices in last collection |

### Resource Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `cloud_resources_total` | Gauge | provider, resource_type, region | Total resources |
| `resource_sync_operations_total` | Counter | provider, status | Sync operations |
| `resource_sync_duration_seconds` | Histogram | provider | Sync duration |
| `estimated_monthly_cost_usd` | Gauge | provider, resource_type | Estimated cost |

### Recommendation Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `recommendations_total` | Gauge | type, status | Total recommendations |
| `potential_savings_usd` | Gauge | provider | Potential savings |
| `recommendations_applied_total` | Counter | type | Applied recommendations |

## Usage Examples

### Recording Pricing Operations

```rust
use observability::PricingMetrics;

// Record list operation
PricingMetrics::record_list_prices("aws", 50, 15);

// Record price creation
PricingMetrics::record_price_created("aws", "compute");

// Record price comparison
PricingMetrics::record_price_comparison(3, 10, 250);

// Set current counts
PricingMetrics::set_pricing_entries_count("aws", 1500);
```

### Using the Timer

```rust
use observability::pricing::PricingTimer;

async fn fetch_prices() {
    let mut timer = PricingTimer::new("fetch", "aws");

    // Do work...

    let duration_ms = timer.stop();
    println!("Fetched in {}ms", duration_ms);
}
```

### Recording Resource Operations

```rust
use observability::ResourceMetrics;

// Set resource counts
ResourceMetrics::set_resources_count("aws", "compute", 150);

// Record sync operations
ResourceMetrics::record_sync_started("aws");
ResourceMetrics::record_sync_completed("aws", 150, 5.5);

// Set cost estimates
ResourceMetrics::set_estimated_monthly_cost("aws", 1500000); // $15,000.00
```

## Prometheus Scrape Config

```yaml
scrape_configs:
  - job_name: 'zerg-api'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Grafana Dashboard

Example queries:

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_errors_total[5m]) / rate(http_requests_total[5m])

# Request latency (p95)
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Pricing entries by provider
pricing_entries_total

# Collection job success rate
sum(rate(price_collection_jobs_total{status="completed"}[1h])) /
sum(rate(price_collection_jobs_total{status=~"completed|failed"}[1h]))

# Potential savings
sum(potential_savings_usd) / 100  # Convert cents to dollars
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     zerg-api                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              metrics_middleware                       │  │
│  │   Records: http_requests_total, duration, errors     │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐   │
│  │  /metrics  │  │  /api/*    │  │  Domain Handlers   │   │
│  │  endpoint  │  │  routes    │  │  PricingMetrics    │   │
│  └────────────┘  └────────────┘  └────────────────────┘   │
│         │                                   │              │
│         └───────────────┬───────────────────┘              │
│                         ▼                                  │
│              ┌────────────────────┐                        │
│              │ Prometheus Recorder │                        │
│              │   (metrics crate)   │                        │
│              └────────────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
              ┌────────────────────┐
              │    Prometheus      │
              │   /metrics scrape  │
              └────────────────────┘
                          │
                          ▼
              ┌────────────────────┐
              │      Grafana       │
              │    Dashboards      │
              └────────────────────┘
```

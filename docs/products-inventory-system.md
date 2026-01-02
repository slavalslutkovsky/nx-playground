# Products/Inventory System

A complete Products/Inventory management system with gRPC API, web storefront, and multi-platform mobile app.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Client Applications                           │
├──────────────────┬──────────────────┬───────────────────────────┤
│  Astro Storefront│  Tauri Mobile    │   Other Clients           │
│  (SSR/SSG Web)   │  (Multi-platform)│                           │
└────────┬─────────┴────────┬─────────┴───────────────────────────┘
         │                  │
         │ HTTP/REST        │ gRPC / Tauri Commands
         ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Products API (Rust)                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                 │
│  │gRPC Server │  │REST Handler│  │Dapr Sidecar│                 │
│  │(Tonic)     │  │(Axum)      │  │            │                 │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘                 │
│        └───────────────┴───────────────┘                        │
│                        │                                         │
│              ┌─────────▼─────────┐                              │
│              │  Product Service  │                              │
│              └─────────┬─────────┘                              │
└────────────────────────┼────────────────────────────────────────┘
                         │
      ┌──────────────────┼──────────────────────┐
      │                  │                      │
┌─────▼─────┐  ┌─────────▼────────┐  ┌─────────▼────────┐
│  MongoDB  │  │      Redis       │  │       NATS       │
│  (Store)  │  │     (Cache)      │  │    (Pub/Sub)     │
└───────────┘  └──────────────────┘  └──────────────────┘
```

## Components

### 1. Products Domain Library (`libs/domains/products`)

A complete domain library following Domain-Driven Design principles:

- **models.rs**: Product entity, CreateProduct, UpdateProduct DTOs, ProductFilter
- **error.rs**: ProductError enum with HTTP status mappings
- **repository.rs**: ProductRepository trait defining data access interface
- **service.rs**: ProductService with business logic and validation
- **mongodb.rs**: MongoDB implementation of ProductRepository
- **handlers.rs**: Axum HTTP handlers with OpenAPI documentation
- **lib.rs**: Module exports and re-exports

### 2. Products Proto (`manifests/grpc/proto/apps/v1/products.proto`)

gRPC service definition with:
- CRUD operations (Create, GetById, UpdateById, DeleteById, List)
- Streaming (ListStream)
- Search operations (Search, GetByCategory, GetBySku, GetByBarcode)
- Inventory operations (UpdateStock, ReserveStock, ReleaseStock, CommitStock, GetLowStock)
- Status operations (Activate, Deactivate, Discontinue)

### 3. Products API (`apps/products-api`)

Rust application exposing both REST and gRPC interfaces:
- REST API on port 3003 (default)
- gRPC server on port 50051 (default)
- OpenAPI/Swagger documentation at `/docs`
- Health checks at `/health` and `/ready`

### 4. Dapr Components (`.dapr/components`)

- **pubsub-nats.yaml**: NATS JetStream pub/sub for events
- **statestore-mongodb.yaml**: MongoDB state store
- **cache-redis.yaml**: Redis cache with TTL

### 5. Astro Storefront (`apps/storefront`)

Server-side rendered web storefront:
- Astro 5.x with Node adapter
- Tailwind CSS 4.x for styling
- Nanostores for cart state
- Pages: Home, Product listing, Product detail, Cart

### 6. Tauri Mobile (`apps/storefront-mobile`)

Cross-platform mobile/desktop app:
- Solid.js for UI
- Tauri 2.0 for native capabilities
- Supports iOS, Android, macOS, Windows, Linux
- Offline-first with local storage

## Getting Started

### Prerequisites

- Rust 1.83+
- Node.js 22+
- Docker & Docker Compose
- MongoDB (via Docker or local)
- Redis (via Docker or local)
- NATS (via Docker or local)

### Local Development

1. **Start infrastructure**:
   ```bash
   docker-compose -f docker-compose.dapr.yml up -d mongodb redis nats
   ```

2. **Run Products API**:
   ```bash
   cd apps/products-api
   cargo run
   ```

3. **Run Astro Storefront**:
   ```bash
   cd apps/storefront
   npm install
   npm run dev
   ```

4. **Run Tauri Mobile (desktop)**:
   ```bash
   cd apps/storefront-mobile
   npm install
   npm run tauri:dev
   ```

### Docker Deployment

```bash
docker-compose -f docker-compose.dapr.yml up -d
```

Services:
- Products API: http://localhost:3003
- Products gRPC: localhost:50051
- Storefront: http://localhost:4321
- MongoDB: localhost:27017
- Redis: localhost:6379
- NATS: localhost:4222

## API Reference

### REST Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/products | List products with filters |
| POST | /api/products | Create a product |
| GET | /api/products/{id} | Get product by ID |
| PUT | /api/products/{id} | Update a product |
| DELETE | /api/products/{id} | Delete a product |
| GET | /api/products/search | Search products |
| GET | /api/products/sku/{sku} | Get by SKU |
| GET | /api/products/barcode/{barcode} | Get by barcode |
| GET | /api/products/category/{category} | Get by category |
| GET | /api/products/count | Count products |
| GET | /api/products/low-stock | Get low stock products |
| POST | /api/products/{id}/stock | Adjust stock |
| POST | /api/products/{id}/reserve | Reserve stock |
| POST | /api/products/{id}/release | Release stock |
| POST | /api/products/{id}/commit | Commit stock |
| POST | /api/products/{id}/activate | Activate product |
| POST | /api/products/{id}/deactivate | Deactivate product |
| POST | /api/products/{id}/discontinue | Discontinue product |

### gRPC Service

See `manifests/grpc/proto/apps/v1/products.proto` for full service definition.

Generate clients:
```bash
cd manifests/grpc
buf generate
```

## Environment Variables

### Products API

| Variable | Default | Description |
|----------|---------|-------------|
| APP_PORT | 3003 | REST API port |
| GRPC_PORT | 50051 | gRPC server port |
| MONGODB_URI | mongodb://localhost:27017/nx_playground | MongoDB connection |
| REDIS_URL | redis://localhost:6379 | Redis connection |
| DAPR_HTTP_PORT | 3500 | Dapr sidecar port |
| RUST_LOG | info | Log level |

### Astro Storefront

| Variable | Default | Description |
|----------|---------|-------------|
| PUBLIC_API_URL | http://localhost:3003 | Products API URL |
| HOST | 0.0.0.0 | Server host |
| PORT | 4321 | Server port |

### Tauri Mobile

| Variable | Default | Description |
|----------|---------|-------------|
| TAURI_API_URL | http://localhost:3003 | Products API URL |

## Development Guide

### Adding a New Endpoint

1. Add method to `ProductRepository` trait
2. Implement in `MongoProductRepository`
3. Add business logic to `ProductService`
4. Add HTTP handler in `handlers.rs`
5. Add gRPC method to proto and `grpc.rs`
6. Update OpenAPI documentation

### Running Tests

```bash
# Domain library tests
cargo test -p domain_products

# API tests
cargo test -p products_api
```

### Building for Production

```bash
# Rust applications
cargo build --release -p products_api

# Astro storefront
cd apps/storefront && npm run build

# Tauri mobile
cd apps/storefront-mobile && npm run tauri:build
```

## Deployment

### Kubernetes

See `k8s/dapr/` for Kubernetes deployment manifests with Dapr integration.

### Cloud Run

Products API can be deployed to Cloud Run:
```bash
gcloud run deploy products-api \
  --source apps/products-api \
  --set-env-vars="MONGODB_URI=..." \
  --allow-unauthenticated
```

## Troubleshooting

### Common Issues

1. **MongoDB connection failed**: Ensure MongoDB is running and accessible
2. **gRPC service not found**: Verify proto was generated with `buf generate`
3. **CORS errors in storefront**: Check API URL configuration
4. **Tauri commands not found**: Rebuild with `npm run tauri:build`

### Logging

Enable debug logging:
```bash
RUST_LOG=debug cargo run -p products_api
```

## License

MIT

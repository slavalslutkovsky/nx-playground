# Cloud Cost Optimization Protos

Protocol Buffer definitions for the multi-cloud cost optimization platform. This library provides gRPC service definitions and message types for pricing, optimization, and resource management across AWS, Azure, and GCP.

## Directory Structure

```
libs/protos/
├── cloud/v1/                    # Proto definitions
│   ├── cost_types.proto         # Shared enums and messages
│   ├── pricing.proto            # PricingService
│   ├── providers.proto          # CloudProviderService
│   ├── optimizer.proto          # CostOptimizerService
│   └── collector.proto          # CollectorService
├── src/
│   ├── lib.rs                   # Library entry point
│   └── generated/               # Generated Rust code (by buf)
│       └── cloud/v1/
│           ├── cloud.v1.rs      # Prost messages
│           └── cloud.v1.tonic.rs # Tonic services
├── buf.yaml                     # Buf module configuration
├── buf.gen.yaml                 # Code generation config
└── Cargo.toml
```

## Services

### PricingService

Query and compare cloud pricing data.

| RPC | Description |
|-----|-------------|
| `GetPrice` | Get a single price entry by ID |
| `ListPrices` | List prices with filtering |
| `ComparePrices` | Compare prices across providers |
| `GetPriceHistory` | Get price history for a SKU |
| `StreamPriceUpdates` | Stream real-time price updates |

### CloudProviderService

Manage cloud provider connections and credentials.

| RPC | Description |
|-----|-------------|
| `RegisterProvider` | Register a new cloud provider |
| `ListProviders` | List registered providers |
| `GetProvider` | Get provider details and status |
| `UpdateProvider` | Update provider configuration |
| `RemoveProvider` | Remove a provider |
| `TestConnection` | Test provider connection |
| `GetProviderHealth` | Get provider health status |

### CostOptimizerService

Generate and manage cost optimization recommendations.

| RPC | Description |
|-----|-------------|
| `GetRecommendations` | List optimization recommendations |
| `GetRecommendation` | Get a specific recommendation |
| `ApplyRecommendation` | Apply a recommendation |
| `DismissRecommendation` | Dismiss a recommendation |
| `GetSavingsSummary` | Get savings summary |
| `AnalyzeResources` | Analyze resources for opportunities |

### CollectorService

Collect pricing data and resource inventory.

| RPC | Description |
|-----|-------------|
| `TriggerPriceCollection` | Start a collection job |
| `GetCollectionStatus` | Get job status |
| `ListCollectionJobs` | List collection jobs |
| `GetInventory` | Get resource inventory |
| `StreamInventoryChanges` | Stream inventory changes |
| `GetCollectionSchedule` | Get collection schedule |
| `UpdateCollectionSchedule` | Update schedule |

## Shared Types

### Enums

```protobuf
enum CloudProvider { AWS, AZURE, GCP }
enum ResourceType { COMPUTE, STORAGE, DATABASE, NETWORK, SERVERLESS, ANALYTICS, KUBERNETES, OTHER }
enum PricingUnit { HOUR, MONTH, GB, GB_HOUR, GB_MONTH, REQUEST, MILLION_REQUESTS, SECOND, UNIT }
enum Currency { USD, EUR, GBP }
```

### Messages

| Message | Description |
|---------|-------------|
| `Money` | Amount in smallest currency unit with precision |
| `TimeRange` | Start/end timestamps for queries |
| `Region` | Region code, display name, and provider |
| `Pagination` | Limit and offset for list requests |
| `ResourceIdentifier` | Cross-cloud resource identifier |

## Usage

### In Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
protos = { workspace = true }
```

Use in your code:

```rust
use protos::cloud::v1::{
    CloudProvider, ResourceType, Money,
    pricing_service_client::PricingServiceClient,
    ListPricesRequest, Pagination,
};

// Create a gRPC client
let mut client = PricingServiceClient::connect("http://[::1]:50051").await?;

// List prices
let request = ListPricesRequest {
    provider: Some(CloudProvider::Aws as i32),
    resource_type: Some(ResourceType::Compute as i32),
    pagination: Some(Pagination { limit: 50, offset: 0 }),
    ..Default::default()
};

let response = client.list_prices(request).await?;
```

### Implementing a gRPC Server

```rust
use protos::cloud::v1::{
    pricing_service_server::{PricingService, PricingServiceServer},
    GetPriceRequest, GetPriceResponse, ListPricesRequest, ListPricesResponse,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MyPricingService;

#[tonic::async_trait]
impl PricingService for MyPricingService {
    async fn get_price(
        &self,
        request: Request<GetPriceRequest>,
    ) -> Result<Response<GetPriceResponse>, Status> {
        // Implementation
        todo!()
    }

    async fn list_prices(
        &self,
        request: Request<ListPricesRequest>,
    ) -> Result<Response<ListPricesResponse>, Status> {
        // Implementation
        todo!()
    }

    // ... other methods
}
```

## Code Generation

### Prerequisites

- [buf CLI](https://buf.build/docs/installation) installed
- Rust toolchain

### Generate Code

```bash
# From project root
just proto-cloud

# Or manually
cd libs/protos
buf generate
```

### Workflow Commands

```bash
just proto-cloud-fmt    # Format proto files
just proto-cloud-lint   # Lint proto files
just proto-cloud-gen    # Generate Rust code
just proto-cloud-check  # Verify Rust compiles
just proto-cloud        # Run all steps
```

## Related Components

| Component | Path | Description |
|-----------|------|-------------|
| Pricing Domain | `libs/domains/pricing/` | Business logic, entities, handlers |
| Pricing API | `apps/zerg/api/` | REST API at `/api/cloud-prices/` |
| TypeScript Types | `libs/domains/pricing/types/` | Generated via ts-rs |
| Database Migration | `libs/migration/` | `m20251216_000000_create_cloud_prices` |

## API Endpoints (REST)

The pricing domain exposes HTTP endpoints via zerg-api:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/cloud-prices/` | List prices |
| POST | `/api/cloud-prices/` | Create price |
| GET | `/api/cloud-prices/{id}` | Get by ID |
| PUT | `/api/cloud-prices/{id}` | Update |
| DELETE | `/api/cloud-prices/{id}` | Delete |
| GET | `/api/cloud-prices/compare` | Compare prices |
| GET | `/api/cloud-prices/stats` | Statistics |

## Recommendation Types

| Type | Description |
|------|-------------|
| `RIGHTSIZING` | Resize over/under-provisioned resources |
| `RESERVED_INSTANCE` | Purchase reserved capacity |
| `SPOT_INSTANCE` | Use spot/preemptible instances |
| `IDLE_RESOURCE` | Terminate idle resources |
| `STORAGE_TIER` | Move to cheaper storage tier |
| `REGION_MIGRATION` | Move to cheaper region |
| `GENERATION_UPGRADE` | Upgrade to newer instance generation |
| `LICENSE_OPTIMIZATION` | Optimize licensing costs |
| `CROSS_CLOUD_MIGRATION` | Move workload to another provider |

## Development

### Adding a New Proto File

1. Create the `.proto` file in `cloud/v1/`
2. Import shared types: `import "v1/cost_types.proto";`
3. Run `just proto-cloud` to generate code
4. Add conversions in `libs/domains/pricing/src/conversions.rs`

### Proto Style Guide

- Use `snake_case` for field names
- Use `PascalCase` for message and enum names
- Prefix enum values with the enum name (e.g., `CLOUD_PROVIDER_UNSPECIFIED`)
- Include `_UNSPECIFIED = 0` as the first enum value
- Use `bytes` for UUIDs (16 bytes)
- Use `int64` for timestamps (Unix seconds)

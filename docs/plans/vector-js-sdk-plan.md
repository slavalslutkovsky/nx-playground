# Vector JS/TS SDK Plan

## Goal
Create a developer-friendly SDK for JS/TS developers to interact with Qdrant vector operations.

## Architecture Decision

### Recommended: Two-Phase Approach

**Phase 1**: TypeScript wrapper over existing gRPC service (immediate value)
**Phase 2**: Optional NAPI-RS core for embedded/edge use cases

---

## Phase 1: TypeScript gRPC Wrapper

### Overview
Wrap the existing `zerg-vector` gRPC service with an ergonomic TypeScript API.

### Location
```
libs/
├── rpc-ts/                    # Generated gRPC types (existing)
│   └── src/generated/
└── vector-sdk/                # NEW: High-level SDK
    ├── package.json
    ├── tsconfig.json
    ├── src/
    │   ├── index.ts
    │   ├── client.ts          # Main VectorClient class
    │   ├── types.ts           # Public types
    │   ├── config.ts          # Configuration
    │   ├── errors.ts          # Custom errors
    │   └── utils/
    │       ├── uuid.ts        # UUID <-> bytes conversion
    │       └── retry.ts       # Retry logic
    └── tests/
```

### Dependencies
```json
{
  "dependencies": {
    "@connectrpc/connect": "^2.0.0",
    "@connectrpc/connect-node": "^2.0.0",
    "@bufbuild/protobuf": "^2.0.0"
  },
  "peerDependencies": {
    "@org/rpc-ts": "workspace:*"
  }
}
```

### API Design

```typescript
// libs/vector-sdk/src/client.ts
import { createClient } from '@connectrpc/connect';
import { VectorService } from '@org/rpc-ts/generated/apps/v1/vector_pb';

export interface VectorClientConfig {
  /** gRPC endpoint (default: from env VECTOR_SERVICE_URL) */
  endpoint?: string;
  /** Project ID for multi-tenancy */
  projectId: string;
  /** Optional namespace */
  namespace?: string;
  /** Default embedding provider */
  defaultProvider?: 'openai' | 'vertexai' | 'cohere' | 'voyage';
  /** Default embedding model */
  defaultModel?: EmbeddingModel;
  /** Request timeout in ms */
  timeout?: number;
}

export class VectorClient {
  private client: Client<typeof VectorService>;
  private config: VectorClientConfig;

  constructor(config: VectorClientConfig) {
    this.config = config;
    this.client = createClient(VectorService, createGrpcTransport({
      baseUrl: config.endpoint ?? process.env.VECTOR_SERVICE_URL,
      httpVersion: '2',
    }));
  }

  // ===== Collection Management =====

  async createCollection(name: string, options?: CreateCollectionOptions): Promise<CollectionInfo> {
    const response = await this.client.createCollection({
      tenant: this.getTenantContext(),
      collectionName: name,
      config: {
        dimension: options?.dimension ?? 1536,
        distance: options?.distance ?? 'cosine',
      },
    });
    return this.mapCollectionInfo(response.collection);
  }

  async deleteCollection(name: string): Promise<void> {
    await this.client.deleteCollection({
      tenant: this.getTenantContext(),
      collectionName: name,
    });
  }

  async getCollection(name: string): Promise<CollectionInfo | null> {
    try {
      const response = await this.client.getCollection({
        tenant: this.getTenantContext(),
        collectionName: name,
      });
      return this.mapCollectionInfo(response.collection);
    } catch (e) {
      if (isNotFoundError(e)) return null;
      throw e;
    }
  }

  async listCollections(): Promise<CollectionInfo[]> {
    const response = await this.client.listCollections({
      tenant: this.getTenantContext(),
    });
    return response.collections.map(this.mapCollectionInfo);
  }

  // ===== Vector Operations =====

  /**
   * Upsert a vector with pre-computed embeddings
   */
  async upsert(collection: string, vector: VectorInput): Promise<string> {
    const response = await this.client.upsert({
      tenant: this.getTenantContext(),
      collectionName: collection,
      vector: {
        id: uuidToBytes(vector.id),
        values: vector.values,
        payload: vector.metadata ? { json: encodeJson(vector.metadata) } : undefined,
      },
      wait: true,
    });
    return vector.id;
  }

  /**
   * Upsert multiple vectors
   */
  async upsertBatch(collection: string, vectors: VectorInput[]): Promise<string[]> {
    const response = await this.client.upsertBatch({
      tenant: this.getTenantContext(),
      collectionName: collection,
      vectors: vectors.map(v => ({
        id: uuidToBytes(v.id),
        values: v.values,
        payload: v.metadata ? { json: encodeJson(v.metadata) } : undefined,
      })),
      wait: true,
    });
    return vectors.map(v => v.id);
  }

  /**
   * Search with pre-computed vector
   */
  async search(collection: string, query: SearchQuery): Promise<SearchResult[]> {
    const response = await this.client.search({
      tenant: this.getTenantContext(),
      collectionName: collection,
      vector: query.vector,
      limit: query.limit ?? 10,
      scoreThreshold: query.scoreThreshold,
      withVectors: query.includeVectors ?? false,
      withPayloads: query.includeMetadata ?? true,
    });
    return response.results.map(this.mapSearchResult);
  }

  /**
   * Get vectors by IDs
   */
  async get(collection: string, ids: string[]): Promise<Vector[]> {
    const response = await this.client.get({
      tenant: this.getTenantContext(),
      collectionName: collection,
      ids: ids.map(uuidToBytes),
      withVectors: true,
      withPayloads: true,
    });
    return response.vectors.map(this.mapVector);
  }

  /**
   * Delete vectors by IDs
   */
  async delete(collection: string, ids: string[]): Promise<void> {
    await this.client.delete({
      tenant: this.getTenantContext(),
      collectionName: collection,
      ids: ids.map(uuidToBytes),
      wait: true,
    });
  }

  // ===== Embedding Operations =====

  /**
   * Generate embeddings for text
   */
  async embed(text: string, options?: EmbedOptions): Promise<number[]> {
    const response = await this.client.embed({
      text,
      provider: options?.provider ?? this.config.defaultProvider ?? 'openai',
      model: options?.model ?? this.config.defaultModel ?? 'text-embedding-3-small',
    });
    return Array.from(response.embedding.values);
  }

  /**
   * Generate embeddings for multiple texts
   */
  async embedBatch(texts: string[], options?: EmbedOptions): Promise<number[][]> {
    const response = await this.client.embedBatch({
      texts,
      provider: options?.provider ?? this.config.defaultProvider ?? 'openai',
      model: options?.model ?? this.config.defaultModel ?? 'text-embedding-3-small',
    });
    return response.embeddings.map(e => Array.from(e.values));
  }

  // ===== Combined Operations (Most Useful for Devs) =====

  /**
   * Upsert document with automatic embedding generation
   * This is the main method JS devs will use
   */
  async upsertDocument(
    collection: string,
    doc: DocumentInput,
    options?: EmbedOptions
  ): Promise<string> {
    const response = await this.client.upsertWithEmbedding({
      tenant: this.getTenantContext(),
      collectionName: collection,
      id: uuidToBytes(doc.id),
      text: doc.content,
      payload: doc.metadata ? { json: encodeJson(doc.metadata) } : undefined,
      provider: options?.provider ?? this.config.defaultProvider ?? 'openai',
      model: options?.model ?? this.config.defaultModel ?? 'text-embedding-3-small',
      wait: true,
    });
    return doc.id;
  }

  /**
   * Semantic search with automatic query embedding
   * This is the main search method JS devs will use
   */
  async searchSimilar(
    collection: string,
    query: string,
    options?: SemanticSearchOptions
  ): Promise<SearchResult[]> {
    const response = await this.client.searchWithEmbedding({
      tenant: this.getTenantContext(),
      collectionName: collection,
      text: query,
      limit: options?.limit ?? 10,
      scoreThreshold: options?.scoreThreshold,
      withVectors: options?.includeVectors ?? false,
      withPayloads: options?.includeMetadata ?? true,
      provider: options?.provider ?? this.config.defaultProvider ?? 'openai',
      model: options?.model ?? this.config.defaultModel ?? 'text-embedding-3-small',
    });
    return response.results.map(this.mapSearchResult);
  }

  /**
   * Get recommendations based on positive/negative examples
   */
  async recommend(
    collection: string,
    options: RecommendOptions
  ): Promise<SearchResult[]> {
    const response = await this.client.recommend({
      tenant: this.getTenantContext(),
      collectionName: collection,
      positiveIds: options.positiveIds.map(uuidToBytes),
      negativeIds: (options.negativeIds ?? []).map(uuidToBytes),
      limit: options.limit ?? 10,
      scoreThreshold: options.scoreThreshold,
      withVectors: options.includeVectors ?? false,
      withPayloads: options.includeMetadata ?? true,
    });
    return response.results.map(this.mapSearchResult);
  }

  // ===== Private Helpers =====

  private getTenantContext() {
    return {
      projectId: uuidToBytes(this.config.projectId),
      namespace: this.config.namespace,
    };
  }
}
```

### Usage Examples

```typescript
// Simple usage for JS developers
import { VectorClient } from '@org/vector-sdk';

const vectors = new VectorClient({
  projectId: 'my-project-uuid',
  defaultProvider: 'openai',
});

// Create a collection
await vectors.createCollection('documents', { dimension: 1536 });

// Index a document (automatic embedding)
await vectors.upsertDocument('documents', {
  id: crypto.randomUUID(),
  content: 'This is my document text that will be embedded automatically',
  metadata: { source: 'wiki', category: 'tech' },
});

// Semantic search (automatic query embedding)
const results = await vectors.searchSimilar('documents', 'find similar documents', {
  limit: 5,
  scoreThreshold: 0.7,
});

// Results include id, score, and metadata
results.forEach(r => console.log(r.id, r.score, r.metadata));
```

### Tasks

- [ ] Generate TypeScript gRPC client from proto (buf generate)
- [ ] Create libs/vector-sdk package structure
- [ ] Implement VectorClient class
- [ ] Add UUID conversion utilities
- [ ] Add error handling and custom error types
- [ ] Add retry logic with exponential backoff
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Add JSDoc documentation
- [ ] Create README with examples

---

## Phase 2: NAPI-RS Core (Optional)

### When to Implement
- Need npm package for external distribution
- Edge/serverless without gRPC access
- Maximum performance requirements
- Offline/embedded use cases

### Location
```
libs/
├── vector-core/               # Rust NAPI crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs
│   │   ├── types.rs
│   │   └── error.rs
│   ├── npm/                   # Platform-specific packages
│   │   ├── darwin-arm64/
│   │   ├── darwin-x64/
│   │   ├── linux-x64-gnu/
│   │   ├── linux-arm64-gnu/
│   │   └── win32-x64-msvc/
│   └── index.d.ts             # TypeScript declarations
└── vector-sdk/                # Updated to use NAPI optionally
```

### Cargo.toml

```toml
[package]
name = "vector-core"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2", features = ["async", "serde-json"] }
napi-derive = "2"
qdrant-client = "1.13"
tokio = { version = "1", features = ["rt-multi-thread"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"

[build-dependencies]
napi-build = "2"

[profile.release]
lto = true
strip = true
```

### Rust Implementation

```rust
// libs/vector-core/src/lib.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod client;
mod types;
mod error;

use client::QdrantClientWrapper;
use types::*;

/// Global tokio runtime for async operations
static RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    })
}

#[napi]
pub struct VectorCore {
    client: Arc<QdrantClientWrapper>,
}

#[napi]
impl VectorCore {
    #[napi(constructor)]
    pub fn new(config: JsVectorConfig) -> Result<Self> {
        let client = get_runtime().block_on(async {
            QdrantClientWrapper::new(config.into()).await
        })?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    #[napi]
    pub async fn create_collection(
        &self,
        name: String,
        dimension: u32,
        distance: Option<String>,
    ) -> Result<JsCollectionInfo> {
        let client = self.client.clone();
        get_runtime()
            .spawn(async move {
                client.create_collection(&name, dimension, distance.as_deref()).await
            })
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map(Into::into)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn upsert(
        &self,
        collection: String,
        id: String,
        values: Vec<f64>,
        payload: Option<serde_json::Value>,
    ) -> Result<String> {
        let client = self.client.clone();
        let values_f32: Vec<f32> = values.into_iter().map(|v| v as f32).collect();

        get_runtime()
            .spawn(async move {
                client.upsert(&collection, &id, values_f32, payload).await
            })
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn search(
        &self,
        collection: String,
        vector: Vec<f64>,
        limit: u32,
        score_threshold: Option<f64>,
    ) -> Result<Vec<JsSearchResult>> {
        let client = self.client.clone();
        let vector_f32: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();

        get_runtime()
            .spawn(async move {
                client.search(&collection, vector_f32, limit, score_threshold.map(|s| s as f32)).await
            })
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map(|results| results.into_iter().map(Into::into).collect())
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete(&self, collection: String, ids: Vec<String>) -> Result<()> {
        let client = self.client.clone();

        get_runtime()
            .spawn(async move {
                client.delete(&collection, ids).await
            })
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}

// Types for JS interop
#[napi(object)]
pub struct JsVectorConfig {
    pub url: String,
    pub api_key: Option<String>,
}

#[napi(object)]
pub struct JsCollectionInfo {
    pub name: String,
    pub vectors_count: i64,
    pub points_count: i64,
}

#[napi(object)]
pub struct JsSearchResult {
    pub id: String,
    pub score: f64,
    pub payload: Option<serde_json::Value>,
}
```

### Build Script (GitHub Actions)

```yaml
# .github/workflows/vector-core-release.yml
name: Build vector-core

on:
  push:
    tags:
      - 'vector-core-v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            npm: darwin-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            npm: darwin-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            npm: linux-x64-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            npm: linux-arm64-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            npm: win32-x64-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          target: ${{ matrix.target }}
      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install dependencies
        run: npm ci
        working-directory: libs/vector-core

      - name: Build
        run: npm run build -- --target ${{ matrix.target }}
        working-directory: libs/vector-core

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: bindings-${{ matrix.npm }}
          path: libs/vector-core/*.node
```

### TypeScript Wrapper with NAPI Fallback

```typescript
// libs/vector-sdk/src/client.ts (updated)
import type { VectorCore as NativeCore } from '@org/vector-core';

export class VectorClient {
  private grpcClient?: GrpcClient;
  private nativeCore?: NativeCore;
  private mode: 'grpc' | 'native';

  constructor(config: VectorClientConfig) {
    if (config.mode === 'native' || !config.endpoint) {
      // Use NAPI native bindings
      const { VectorCore } = require('@org/vector-core');
      this.nativeCore = new VectorCore({
        url: config.qdrantUrl ?? 'http://localhost:6334',
        apiKey: config.qdrantApiKey,
      });
      this.mode = 'native';
    } else {
      // Use gRPC client
      this.grpcClient = createGrpcClient(config);
      this.mode = 'grpc';
    }
  }

  async search(collection: string, query: SearchQuery): Promise<SearchResult[]> {
    if (this.mode === 'native') {
      return this.nativeCore!.search(
        collection,
        query.vector,
        query.limit ?? 10,
        query.scoreThreshold
      );
    } else {
      return this.grpcClient!.search(collection, query);
    }
  }

  // ... other methods with similar pattern
}
```

### Tasks

- [ ] Set up napi-rs project structure
- [ ] Implement Rust core with qdrant-client
- [ ] Handle tokio runtime lifecycle
- [ ] Create platform-specific npm packages
- [ ] Set up GitHub Actions for multi-platform builds
- [ ] Add TypeScript type definitions
- [ ] Update vector-sdk to support both modes
- [ ] Test on all platforms
- [ ] Publish to npm

---

## Implementation Order

### Phase 1 (Recommended Start)
1. Generate TypeScript types from proto
2. Create vector-sdk package
3. Implement VectorClient with gRPC
4. Add utilities and error handling
5. Write tests and documentation

### Phase 2 (When Needed)
1. Set up napi-rs project
2. Implement native Qdrant wrapper
3. Set up multi-platform CI/CD
4. Add fallback logic to vector-sdk
5. Publish native packages

---

## Comparison Summary

| Aspect | Phase 1 (gRPC) | Phase 2 (NAPI) |
|--------|---------------|----------------|
| Setup time | ~1 week | ~2-3 weeks |
| Maintenance | Low | Medium |
| Performance | Good (~5ms) | Best (~1ms) |
| Distribution | Easy | Complex |
| Team can modify | Yes | Partial |
| Offline support | No | Yes |
| Edge/Serverless | Limited | Yes |

## Recommendation

Start with **Phase 1** immediately - it provides 90% of the value with 20% of the effort. Your JS team can start using it right away.

Only proceed to **Phase 2** if you need:
- npm distribution to external teams
- Edge/serverless deployment
- Sub-millisecond performance requirements

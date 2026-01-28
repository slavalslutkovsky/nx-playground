# Terran Web App Implementation Plan

## Overview

Create a SolidJS-based web application at `apps/terran/web` with full-stack capabilities, integrating with vector and tasks gRPC services via both direct gRPC (server-side) and HTTP/REST (client-side).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    apps/terran/web                          │
│  ┌─────────────────────┐  ┌─────────────────────────────┐  │
│  │   SolidStart SSR    │  │      Astro Islands          │  │
│  │   (Server Routes)   │  │   (Static + Interactive)    │  │
│  └──────────┬──────────┘  └──────────────┬──────────────┘  │
│             │                            │                  │
│             ▼                            ▼                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Connect-ES gRPC Client                  │   │
│  │         (Server-side, direct to services)            │   │
│  └──────────────────────────┬──────────────────────────┘   │
│                             │                               │
└─────────────────────────────┼───────────────────────────────┘
                              │ gRPC (H2)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      apps/zerg/api                          │
│  ┌─────────────────┐  ┌─────────────────────────────────┐  │
│  │  /api/vectors   │  │   /api/vectors-direct           │  │
│  │  (gRPC proxy)   │  │   (Direct Qdrant)               │  │
│  └────────┬────────┘  └─────────────────────────────────┘  │
│           │ gRPC                                            │
└───────────┼─────────────────────────────────────────────────┘
            ▼
┌───────────────────────┐  ┌───────────────────────┐
│   apps/zerg/vector    │  │   apps/zerg/tasks     │
│   (Vector Service)    │  │   (Tasks Service)     │
└───────────────────────┘  └───────────────────────┘
```

## Implementation Steps

### Phase 1: Fix Current Rust Warnings

1. Clean up unused imports in `libs/domains/vector/src/handlers/direct.rs`
2. Clean up unused imports in `libs/domains/vector/src/handlers/mod.rs`

### Phase 2: Update zerg-api for Multi-Service Support

1. **Add vector routes to zerg-api** (mirror tasks pattern)
   - `/api/vectors` - gRPC proxy routes
   - `/api/vectors-direct` - Direct Qdrant routes
   - Files to modify:
     - `apps/zerg/api/src/api/mod.rs` - Add vector routes
     - `apps/zerg/api/src/api/vectors.rs` - gRPC proxy handlers
     - `apps/zerg/api/src/api/vectors_direct.rs` - Direct handlers
     - `apps/zerg/api/Cargo.toml` - Add vector domain dependency

2. **Multi-service gRPC client management**
   - Create unified gRPC pool for vector and tasks services
   - Update `apps/zerg/api/src/grpc_pool.rs` for multi-service support
   - Update `apps/zerg/api/src/state.rs` with vector client

### Phase 3: Create SolidStart App at apps/terran/web

1. **Initialize SolidStart project**
   - `apps/terran/web/package.json` with dependencies
   - `apps/terran/web/app.config.ts` - Vinxi/SolidStart config
   - `apps/terran/web/tsconfig.json`

2. **Project structure**
   ```
   apps/terran/web/
   ├── package.json
   ├── app.config.ts
   ├── tsconfig.json
   ├── src/
   │   ├── app.tsx              # Root component
   │   ├── entry-client.tsx     # Client entry
   │   ├── entry-server.tsx     # Server entry
   │   ├── routes/
   │   │   ├── index.tsx        # Dashboard
   │   │   ├── vectors/
   │   │   │   ├── index.tsx    # Vector search
   │   │   │   └── [id].tsx     # Vector details
   │   │   └── tasks/
   │   │       ├── index.tsx    # Task list
   │   │       └── [id].tsx     # Task details
   │   ├── lib/
   │   │   ├── grpc-client.ts   # Server-side gRPC (Connect-ES)
   │   │   ├── rest-client.ts   # Client-side REST
   │   │   └── queries.ts       # TanStack Query definitions
   │   └── components/
   │       ├── SearchBox.tsx
   │       ├── VectorResults.tsx
   │       └── TaskList.tsx
   └── public/
   ```

3. **Server-side gRPC client (Connect-ES)**
   - Use `@connectrpc/connect` with `@connectrpc/connect-node`
   - Import from `@nx-playground/rpc-ts`
   - Server functions call gRPC directly

4. **Client-side REST with TanStack Query**
   - Use `@tanstack/solid-query`
   - Fetch from zerg-api HTTP endpoints
   - Automatic caching and refetching

### Phase 4: Add Astro Integration (Optional BFF Features)

1. **Create Astro project at apps/terran/web-astro** (or integrate)
   - Use `@astrojs/solid-js` for Solid components
   - Static pages with interactive islands
   - API routes for BFF patterns

### Phase 5: Full OpenTelemetry Integration

1. **TypeScript/Node.js tracing**
   - Add `@opentelemetry/sdk-node`
   - Add `@opentelemetry/auto-instrumentations-node`
   - Add `@opentelemetry/exporter-trace-otlp-grpc`
   - Configure in `src/instrumentation.ts`

2. **Trace context propagation**
   - Pass trace-id from client → server → gRPC services
   - Use `traceparent` header in HTTP requests
   - gRPC metadata for trace context

3. **Metrics collection**
   - Request duration histograms
   - Error rates
   - gRPC method latencies

4. **Connect to existing infrastructure**
   - Zerg services already use `libs/core/config/src/tracing.rs`
   - Interceptors in `libs/core/grpc/src/interceptors/`
   - Ensure consistent service names and trace ID propagation

## Dependencies

### apps/terran/web/package.json
```json
{
  "dependencies": {
    "@solidjs/router": "^0.15.x",
    "@solidjs/start": "^1.x",
    "solid-js": "^1.9.x",
    "@tanstack/solid-query": "^5.x",
    "@connectrpc/connect": "^2.0.0",
    "@connectrpc/connect-node": "^2.0.0",
    "@nx-playground/rpc-ts": "workspace:*",
    "@opentelemetry/sdk-node": "^0.57.x",
    "@opentelemetry/auto-instrumentations-node": "^0.55.x",
    "@opentelemetry/exporter-trace-otlp-grpc": "^0.57.x"
  }
}
```

### Rust dependencies (already in workspace)
- `libs/domains/vector` - Vector domain library
- `libs/core/grpc` - gRPC utilities and interceptors
- `libs/core/config` - Tracing configuration

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `libs/domains/vector/src/handlers/direct.rs` | Edit | Fix unused imports |
| `libs/domains/vector/src/handlers/mod.rs` | Edit | Fix unused imports |
| `apps/zerg/api/src/api/mod.rs` | Edit | Add vector routes |
| `apps/zerg/api/src/api/vectors.rs` | Create | gRPC proxy handlers |
| `apps/zerg/api/src/grpc_pool.rs` | Edit | Multi-service support |
| `apps/zerg/api/src/state.rs` | Edit | Add vector client |
| `apps/terran/web/*` | Create | New SolidStart app |

## Implementation Order

1. Fix Rust warnings (quick cleanup)
2. Add vector routes to zerg-api (backend ready)
3. Create SolidStart app skeleton
4. Implement gRPC client for server-side
5. Implement REST client with TanStack Query
6. Add OpenTelemetry instrumentation
7. (Optional) Add Astro for additional BFF features

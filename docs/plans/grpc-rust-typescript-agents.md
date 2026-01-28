# gRPC: Rust Services with TypeScript/JS Agent Clients

This document describes the architecture and patterns for building high-performance Rust gRPC services consumed by TypeScript/JavaScript agents.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Why This Stack](#why-this-stack)
3. [Project Structure](#project-structure)
4. [Proto Definition](#proto-definition)
5. [Code Generation with Buf](#code-generation-with-buf)
6. [Rust Server Implementation](#rust-server-implementation)
7. [TypeScript Client Usage](#typescript-client-usage)
8. [Agent Integration](#agent-integration)
9. [Testing](#testing)
10. [Deployment](#deployment)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Agent Layer (TypeScript)                          │
│                                                                          │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐              │
│   │  LangGraph   │    │  Google ADK  │    │     A2A      │              │
│   │  Supervisor  │    │    Agent     │    │   Protocol   │              │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘              │
│          │                   │                   │                       │
│          └───────────────────┼───────────────────┘                       │
│                              │                                           │
│                   ┌──────────▼──────────┐                                │
│                   │   Agent Tools       │                                │
│                   │  (gRPC Wrappers)    │                                │
│                   └──────────┬──────────┘                                │
│                              │                                           │
│                   ┌──────────▼──────────┐                                │
│                   │  @nx-playground/    │  Generated from proto          │
│                   │     rpc-ts          │  via buf + connect-es          │
│                   └──────────┬──────────┘                                │
└──────────────────────────────┼───────────────────────────────────────────┘
                               │
                               │ gRPC / HTTP/2
                               │ (Connect Protocol)
                               │
┌──────────────────────────────┼───────────────────────────────────────────┐
│                              ▼                                           │
│                   ┌─────────────────────┐                                │
│                   │   Rust gRPC Server  │  tonic + prost                 │
│                   │   (High Performance)│                                │
│                   └──────────┬──────────┘                                │
│                              │                                           │
│    ┌─────────────────────────┼─────────────────────────────┐             │
│    │                         ▼                             │             │
│    │  ┌────────────┐  ┌────────────┐  ┌────────────┐      │             │
│    │  │  Vector    │  │   Tasks    │  │   Users    │      │             │
│    │  │  Service   │  │  Service   │  │  Service   │      │             │
│    │  │ (Qdrant)   │  │  (CRUD)    │  │  (Auth)    │      │             │
│    │  └────────────┘  └────────────┘  └────────────┘      │             │
│    │                                                       │             │
│    │              Rust Service Layer                       │             │
│    └───────────────────────────────────────────────────────┘             │
│                              │                                           │
│    ┌─────────────────────────┼─────────────────────────────┐             │
│    │                         ▼                             │             │
│    │  ┌────────────┐  ┌────────────┐  ┌────────────┐      │             │
│    │  │  Qdrant    │  │ PostgreSQL │  │   Redis    │      │             │
│    │  │  (Vector)  │  │   (SQL)    │  │  (Cache)   │      │             │
│    │  └────────────┘  └────────────┘  └────────────┘      │             │
│    │                                                       │             │
│    │              Data Layer                               │             │
│    └───────────────────────────────────────────────────────┘             │
│                                                                          │
│                        Backend Layer (Rust)                              │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Why This Stack

### TypeScript for Agents

| Advantage | Details |
|-----------|---------|
| **Rich LLM Ecosystem** | LangChain, LangGraph, Google ADK, Vercel AI SDK |
| **Rapid Iteration** | Hot reload, dynamic typing when needed |
| **Better DX** | Extensive tooling, npm ecosystem |
| **Prompt Engineering** | Easier string manipulation, templating |

### Rust for Services

| Advantage | Details |
|-----------|---------|
| **Performance** | 10-100x faster than Node.js for compute |
| **Memory Efficiency** | No GC pauses, predictable latency |
| **Type Safety** | Compile-time guarantees |
| **Concurrency** | async/await + tokio = massive throughput |

### gRPC for Communication

| Advantage | Details |
|-----------|---------|
| **Schema-First** | Single source of truth (proto files) |
| **Type Safety** | Generated clients/servers match exactly |
| **Performance** | Binary protocol, HTTP/2, multiplexing |
| **Streaming** | Server, client, and bidirectional streaming |
| **Compression** | Built-in Zstd/gzip support |

---

## Project Structure

```
nx-playground/
├── manifests/grpc/
│   ├── buf.yaml              # Buf module configuration
│   ├── buf.gen.yaml          # Code generation config
│   └── proto/apps/v1/        # Proto definitions
│       ├── common.proto      # Shared types
│       ├── users.proto       # User service
│       ├── tasks.proto       # Tasks service (optimized)
│       ├── vector.proto      # Vector search service
│       └── terran.proto      # Code graph service
│
├── libs/
│   ├── rpc/                  # Rust generated code
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── generated/    # prost + tonic output
│   │           ├── tasks/
│   │           ├── users/
│   │           └── vector/
│   │
│   ├── rpc-ts/               # TypeScript generated code
│   │   ├── package.json
│   │   └── src/
│   │       ├── index.ts      # Exports
│   │       ├── utils.ts      # UUID/payload helpers
│   │       └── generated/    # protobuf-es + connect-es output
│   │           └── apps/v1/
│   │               ├── tasks_pb.ts
│   │               ├── tasks_connect.ts
│   │               └── ...
│   │
│   └── core/grpc/            # Rust gRPC utilities
│       └── src/
│           ├── channel/      # Channel creation
│           ├── interceptors/ # Auth, tracing, metrics
│           └── client/       # Client configuration
│
├── apps/
│   ├── zerg/                 # Rust services
│   │   ├── tasks/            # Tasks gRPC server
│   │   ├── vector/           # Vector gRPC server
│   │   └── api/              # REST gateway (uses gRPC clients)
│   │
│   └── agents/               # TypeScript agents
│       ├── agent-patterns/   # [EXAMPLE] Communication patterns
│       └── ...
```

---

## Proto Definition

### Best Practices

```protobuf
// manifests/grpc/proto/apps/v1/example.proto
syntax = "proto3";

package example.v1;

// 1. Use package versioning (v1, v2, etc.)
// 2. Optimize for binary efficiency where it matters

// Binary-optimized ID (16 bytes vs 36-char string)
message TaskId {
  bytes id = 1;  // UUID as bytes
}

// Use enums for bounded values (1 byte vs string)
enum Priority {
  PRIORITY_UNSPECIFIED = 0;
  PRIORITY_LOW = 1;
  PRIORITY_MEDIUM = 2;
  PRIORITY_HIGH = 3;
  PRIORITY_URGENT = 4;
}

// Unix timestamps (8 bytes vs 24+ char ISO string)
message Task {
  bytes id = 1;
  string title = 2;
  string description = 3;
  Priority priority = 4;
  int64 created_at = 5;  // Unix timestamp
  int64 updated_at = 6;
}

// Service definition
service TaskService {
  // Unary RPC
  rpc Create(CreateRequest) returns (CreateResponse);
  rpc GetById(GetByIdRequest) returns (GetByIdResponse);

  // Server streaming for large result sets
  rpc ListStream(ListStreamRequest) returns (stream Task);
}

message CreateRequest {
  string title = 1;
  string description = 2;
  Priority priority = 3;
}

message CreateResponse {
  Task task = 1;
}

message GetByIdRequest {
  bytes id = 1;
}

message GetByIdResponse {
  Task task = 1;
}

message ListStreamRequest {
  int32 limit = 1;
  int32 offset = 2;
}
```

---

## Code Generation with Buf

### Configuration

**buf.yaml** (Module configuration):
```yaml
version: v2
modules:
  - path: proto
lint:
  use:
    - STANDARD
  except:
    - PACKAGE_DIRECTORY_MATCH
breaking:
  use:
    - FILE
```

**buf.gen.yaml** (Generation configuration):
```yaml
version: v2
managed:
  enabled: true
plugins:
  # Rust: Prost message types
  - remote: buf.build/community/neoeinstein-prost:v0.5.0
    out: ../../libs/rpc/src/generated
    opt:
      - prost=0.14.1

  # Rust: Tonic gRPC stubs
  - remote: buf.build/community/neoeinstein-tonic:v0.5.0
    out: ../../libs/rpc/src/generated
    opt:
      - tonic=0.14.2

  # TypeScript: Protobuf-ES message types
  - remote: buf.build/bufbuild/es:v2.2.3
    out: ../../libs/rpc-ts/src/generated
    opt:
      - target=ts

  # TypeScript: Connect-ES gRPC clients
  - remote: buf.build/connectrpc/es:v1.6.1
    out: ../../libs/rpc-ts/src/generated
    opt:
      - target=ts
```

### Generate Code

```bash
# From manifests/grpc directory
buf generate

# Or via nx
bun nx run rpc:generate
```

---

## Rust Server Implementation

### Service Implementation

```rust
// apps/zerg/tasks/src/service.rs
use tonic::{Request, Response, Status};
use rpc::tasks::{
    tasks_service_server::TasksService,
    CreateRequest, CreateResponse,
    GetByIdRequest, GetByIdResponse,
    ListStreamRequest, Task,
};
use tokio_stream::wrappers::ReceiverStream;

pub struct TasksServiceImpl {
    db: DatabasePool,
}

#[tonic::async_trait]
impl TasksService for TasksServiceImpl {
    // Unary RPC
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        let task = self.db.create_task(
            &req.title,
            &req.description,
            req.priority,
        ).await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateResponse { task: Some(task) }))
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        let id = uuid::Uuid::from_slice(&request.into_inner().id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        let task = self.db.get_task(id).await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetByIdResponse { task: Some(task) }))
    }

    // Server streaming RPC
    type ListStreamStream = ReceiverStream<Result<Task, Status>>;

    async fn list_stream(
        &self,
        request: Request<ListStreamRequest>,
    ) -> Result<Response<Self::ListStreamStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        let db = self.db.clone();
        tokio::spawn(async move {
            let tasks = db.list_tasks(req.limit, req.offset).await;
            for task in tasks {
                if tx.send(Ok(task)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

### Server Setup

```rust
// apps/zerg/tasks/src/main.rs
use tonic::transport::Server;
use rpc::tasks::tasks_service_server::TasksServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse()?;
    let service = TasksServiceImpl::new().await?;

    println!("Tasks gRPC server listening on {}", addr);

    Server::builder()
        // Enable Zstd compression
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
        // Add service
        .add_service(TasksServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

---

## TypeScript Client Usage

### Basic Client Setup

```typescript
// apps/agents/example/src/grpc-client.ts
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { TaskService } from '@nx-playground/rpc-ts';

// Create transport (reusable)
const transport = createGrpcTransport({
  baseUrl: 'http://localhost:50051',
  httpVersion: '2',
});

// Create typed client
const tasksClient = createClient(TaskService, transport);

// Use the client
async function createTask() {
  const response = await tasksClient.create({
    title: 'My Task',
    description: 'Task description',
    priority: Priority.HIGH,
  });

  console.log('Created task:', response.task);
  return response.task;
}

async function getTask(id: Uint8Array) {
  const response = await tasksClient.getById({ id });
  return response.task;
}

// Server streaming
async function listTasks() {
  const tasks: Task[] = [];

  for await (const task of tasksClient.listStream({ limit: 100, offset: 0 })) {
    tasks.push(task);
  }

  return tasks;
}
```

### UUID Conversion Utilities

```typescript
// libs/rpc-ts/src/utils.ts
export function uuidToBytes(uuid: string): Uint8Array {
  const hex = uuid.replace(/-/g, '');
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

export function bytesToUuid(bytes: Uint8Array): string {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

// Usage
const id = uuidToBytes('550e8400-e29b-41d4-a716-446655440000');
const task = await tasksClient.getById({ id });
```

---

## Agent Integration

### Creating Agent Tools from gRPC Clients

```typescript
// apps/agents/example/src/tools/grpc-tools.ts
import { tool } from '@langchain/core/tools';
import { z } from 'zod';
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { TaskService, VectorService } from '@nx-playground/rpc-ts';
import { uuidToBytes } from '@nx-playground/rpc-ts';

// Setup clients
const transport = createGrpcTransport({
  baseUrl: process.env.GRPC_URL || 'http://localhost:50051',
  httpVersion: '2',
});

const tasksClient = createClient(TaskService, transport);
const vectorClient = createClient(VectorService, transport);

// Tool: Create Task
export const createTaskTool = tool(
  async ({ title, description, priority }) => {
    const response = await tasksClient.create({
      title,
      description,
      priority: priority === 'high' ? 3 : priority === 'medium' ? 2 : 1,
    });
    return JSON.stringify(response.task);
  },
  {
    name: 'create_task',
    description: 'Create a new task in the task management system',
    schema: z.object({
      title: z.string().describe('Task title'),
      description: z.string().describe('Task description'),
      priority: z.enum(['low', 'medium', 'high']).describe('Task priority'),
    }),
  },
);

// Tool: Search Vectors
export const searchVectorsTool = tool(
  async ({ query, collection, limit }) => {
    const response = await vectorClient.searchWithEmbedding({
      collection,
      text: query,
      limit,
      embeddingProvider: 'OPENAI',
      embeddingModel: 'text-embedding-3-small',
    });

    return JSON.stringify(response.results.map(r => ({
      id: r.id,
      score: r.score,
      payload: r.payload,
    })));
  },
  {
    name: 'search_vectors',
    description: 'Search for similar documents in a vector collection',
    schema: z.object({
      query: z.string().describe('Search query text'),
      collection: z.string().describe('Collection name to search'),
      limit: z.number().default(10).describe('Maximum results to return'),
    }),
  },
);

// Export all tools
export const GRPC_TOOLS = [createTaskTool, searchVectorsTool];
```

### Using Tools in LangGraph Agent

```typescript
// apps/agents/example/src/agent.ts
import { createReactAgent } from '@langchain/langgraph/prebuilt';
import { ChatAnthropic } from '@langchain/anthropic';
import { GRPC_TOOLS } from './tools/grpc-tools.js';

const llm = new ChatAnthropic({
  model: 'claude-sonnet-4-20250514',
});

export const agent = createReactAgent({
  llm,
  tools: GRPC_TOOLS,
});

// Usage
const result = await agent.invoke({
  messages: [
    {
      role: 'user',
      content: 'Create a high priority task to review the PR and search for related documentation',
    },
  ],
});
```

### Using Tools in Google ADK Agent

```typescript
// apps/agents/example/src/adk-agent.ts
import { Agent, defineTool } from '@google/adk';
import { z } from 'zod';
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { TaskService } from '@nx-playground/rpc-ts';

const transport = createGrpcTransport({
  baseUrl: 'http://localhost:50051',
  httpVersion: '2',
});
const tasksClient = createClient(TaskService, transport);

// Define ADK tool
const createTaskTool = defineTool({
  name: 'create_task',
  description: 'Create a new task',
  inputSchema: z.object({
    title: z.string(),
    description: z.string(),
    priority: z.enum(['low', 'medium', 'high']),
  }),
  handler: async ({ title, description, priority }) => {
    const response = await tasksClient.create({
      title,
      description,
      priority: priority === 'high' ? 3 : 2,
    });
    return `Created task: ${response.task?.title}`;
  },
});

// Create agent
export const agent = new Agent({
  name: 'task-manager',
  model: 'gemini-2.0-flash',
  tools: [createTaskTool],
  instructions: 'You help manage tasks using gRPC services.',
});
```

---

## Testing

### Unit Testing gRPC Tools

```typescript
// apps/agents/example/src/__tests__/grpc-tools.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { createTaskTool, searchVectorsTool } from '../tools/grpc-tools.js';

describe('gRPC Tools', () => {
  // These tests require running gRPC servers
  // Use docker-compose to start them before running tests

  describe('createTaskTool', () => {
    it('should create a task', async () => {
      const result = await createTaskTool.invoke({
        title: 'Test Task',
        description: 'Test description',
        priority: 'high',
      });

      const task = JSON.parse(result);
      expect(task.title).toBe('Test Task');
      expect(task.priority).toBe(3);
    });
  });

  describe('searchVectorsTool', () => {
    it('should search vectors', async () => {
      const result = await searchVectorsTool.invoke({
        query: 'authentication',
        collection: 'docs',
        limit: 5,
      });

      const results = JSON.parse(result);
      expect(Array.isArray(results)).toBe(true);
    });
  });
});
```

### Integration Testing

```typescript
// apps/agents/example/src/__tests__/integration.test.ts
import { describe, it, expect } from 'vitest';
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { TaskService } from '@nx-playground/rpc-ts';
import { uuidToBytes, bytesToUuid } from '@nx-playground/rpc-ts';

describe('gRPC Integration', () => {
  const transport = createGrpcTransport({
    baseUrl: 'http://localhost:50051',
    httpVersion: '2',
  });
  const client = createClient(TaskService, transport);

  it('should perform full CRUD cycle', async () => {
    // Create
    const createRes = await client.create({
      title: 'Integration Test Task',
      description: 'Testing CRUD',
      priority: 2,
    });
    expect(createRes.task).toBeDefined();
    const taskId = createRes.task!.id;

    // Read
    const getRes = await client.getById({ id: taskId });
    expect(getRes.task?.title).toBe('Integration Test Task');

    // Update
    const updateRes = await client.updateById({
      id: taskId,
      title: 'Updated Title',
    });
    expect(updateRes.task?.title).toBe('Updated Title');

    // Delete
    await client.deleteById({ id: taskId });

    // Verify deleted
    await expect(client.getById({ id: taskId }))
      .rejects.toThrow();
  });

  it('should handle streaming', async () => {
    const tasks: unknown[] = [];

    for await (const task of client.listStream({ limit: 10, offset: 0 })) {
      tasks.push(task);
    }

    expect(Array.isArray(tasks)).toBe(true);
  });
});
```

---

## Deployment

### Docker Compose (Development)

```yaml
# manifests/dockers/compose.yaml
services:
  tasks-grpc:
    build:
      context: ../..
      dockerfile: apps/zerg/tasks/Dockerfile
    ports:
      - "50051:50051"
    environment:
      - DATABASE_URL=postgres://user:pass@postgres:5432/tasks
    depends_on:
      - postgres

  vector-grpc:
    build:
      context: ../..
      dockerfile: apps/zerg/vector/Dockerfile
    ports:
      - "50052:50052"
    environment:
      - QDRANT_URL=http://qdrant:6334
    depends_on:
      - qdrant

  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
      POSTGRES_DB: tasks

  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
      - "6334:6334"
```

### Kubernetes (Production)

```yaml
# k8s/tasks-grpc/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tasks-grpc
  labels:
    app: tasks-grpc
spec:
  replicas: 3
  selector:
    matchLabels:
      app: tasks-grpc
  template:
    metadata:
      labels:
        app: tasks-grpc
    spec:
      containers:
        - name: tasks-grpc
          image: gcr.io/project/tasks-grpc:latest
          ports:
            - containerPort: 50051
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 5
          readinessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: tasks-grpc
spec:
  selector:
    app: tasks-grpc
  ports:
    - port: 50051
      targetPort: 50051
  type: ClusterIP
```

---

## Quick Reference

### Generate Code
```bash
cd manifests/grpc && buf generate
```

### Start Rust Server
```bash
cargo run -p tasks-grpc
# or
bun nx serve zerg-tasks
```

### Test TypeScript Client
```bash
bun nx test agent-patterns
```

### Key Dependencies

**Rust (Cargo.toml)**:
```toml
prost = "0.14.1"
tonic = { version = "0.14.2", features = ["zstd"] }
```

**TypeScript (package.json)**:
```json
{
  "@bufbuild/protobuf": "^2.2.3",
  "@connectrpc/connect": "^2.0.0",
  "@connectrpc/connect-node": "^2.0.0"
}
```

---

## Summary

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Proto** | Protobuf | Contract definition |
| **Generation** | Buf | Multi-language code gen |
| **Rust Server** | Tonic + Prost | High-performance services |
| **TS Client** | Connect-ES | Type-safe gRPC calls |
| **Agent Tools** | LangChain/ADK | LLM integration |

This architecture provides:
- **Type safety** across the entire stack
- **High performance** where it matters (Rust)
- **Developer velocity** where it matters (TypeScript)
- **Single source of truth** (proto files)

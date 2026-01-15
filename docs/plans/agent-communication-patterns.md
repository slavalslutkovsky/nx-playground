# Agent Communication Patterns

This document compares five approaches for agent-to-agent communication in a multi-agent system.

## Architecture Context

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENTS                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │  TanStack    │    │    Astro     │    │  AI Agents   │                 │
│   │  (React)     │    │   (Static)   │    │ (TypeScript) │                 │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                 │
│          │                   │                   │                          │
│          │ TS gRPC Client    │ TS gRPC Client    │ TS gRPC Client          │
│          │ (Connect)         │ (Connect)         │ (Connect)               │
│          └───────────────────┴───────────────────┘                          │
│                              │                                               │
│                      ┌───────┴───────┐                                      │
│                      │   REST API    │  (Alternative: HTTP/JSON)            │
│                      └───────┬───────┘                                      │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REST-to-gRPC PROXY                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                      zerg-api (Axum :8080)                            │  │
│   │                                                                       │  │
│   │   • REST endpoints for web clients                                   │  │
│   │   • Rust gRPC clients (TasksServiceClient, VectorServiceClient)      │  │
│   │   • Proxies REST requests to gRPC services                           │  │
│   │   • JWT auth, sessions, rate limiting                                │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                              │                                               │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │ gRPC (Rust client)
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          gRPC SERVERS (Rust)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│   │   zerg-tasks    │    │   zerg-vector   │    │  (future svc)   │        │
│   │   (Tonic)       │    │   (Tonic)       │    │                 │        │
│   │   :50051        │    │   :50052        │    │   :50053        │        │
│   └────────┬────────┘    └────────┬────────┘    └─────────────────┘        │
│            │                      │                                         │
│            ▼                      ▼                                         │
│       PostgreSQL              Qdrant                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Client Types

| Client | Language | Use Case | Transport |
|--------|----------|----------|-----------|
| **TanStack/Astro** | TypeScript | Web frontends | gRPC (Connect) or REST |
| **AI Agents** | TypeScript | LangGraph agents | gRPC (Connect) or REST |
| **zerg-api** | Rust | REST-to-gRPC proxy | gRPC (Tonic) |
| **CLI tools** | Rust/TS | Developer utilities | gRPC |

### Key Points

1. **AI Agents are TypeScript** - Built with LangGraph/LangChain
2. **gRPC Servers are Rust** - High performance with Tonic
3. **TS clients use Connect protocol** - Works in browser and Node.js
4. **zerg-api is a proxy** - REST for simple clients, gRPC internally
5. **Fullstack apps have choice** - Direct gRPC or REST via proxy

---

## Overview

| Pattern | Transport | Coupling | Scaling | Best For |
|---------|-----------|----------|---------|----------|
| **HTTP** | REST/JSON | Medium | Good | Simple, direct calls |
| **gRPC** | Protobuf/HTTP2 | Medium | Excellent | Typed contracts, streaming, Rust integration |
| **NATS** | Message Queue | Loose | Excellent | Async, resilient systems |
| **RemoteRunnable** | HTTP + LangServe | Medium | Good | LangGraph-native |
| **A2A** | JSON-RPC/HTTP | Loose | Excellent | Cross-vendor interop |

## Current Agent Structure

```
apps/agents/
├── rag-agent/           # RAG retrieval (StateGraph)
│   └── src/retrieval_graph/graph.ts
│       export const graph = builder.compile()
│
├── code-tester/         # Memory agent (StateGraph)
│   └── src/memory_agent/graph.ts
│       export const graph = builder.compile()
│
├── whatsup-agent/       # ReAct agent (createAgent)
│   └── src/agent.ts
│       export const agent = createAgent({...})
│
└── supervisor-*/        # Orchestrators
```

---

## Pattern 1: NATS (Message Queue)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     NATS JetStream                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Stream: AGENTS                                          │    │
│  │  Subjects: agents.{name}.request, agents.{name}.response │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │ publish            │ subscribe          │ subscribe
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │Supervisor│         │RAG Agent│         │Memory   │
    │         │         │ Worker  │         │ Worker  │
    └─────────┘         └─────────┘         └─────────┘
```

### Flow

```
1. Supervisor publishes to: agents.rag.request
   {
     "requestId": "uuid",
     "replyTo": "agents.supervisor.response",
     "payload": { "query": "search auth docs" }
   }

2. RAG Agent subscribes to: agents.rag.request
   - Processes request
   - Publishes result to: agents.supervisor.response

3. Supervisor receives response on: agents.supervisor.response
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Decoupled - agents don't know each other | Extra infrastructure (NATS server) |
| Auto load balancing via queue groups | Slightly higher latency (broker hop) |
| Resilience - broker buffers if agent down | More complex debugging |
| Easy to scale - just add workers | Message serialization overhead |
| Built-in retry/DLQ support | |

### When to Use

- High availability requirements
- Need independent scaling per agent
- Async/fire-and-forget patterns
- Multiple instances of same agent
- You already have NATS infrastructure

---

## Pattern 2: HTTP (REST API)

### Architecture

```
┌─────────────┐     POST /invoke      ┌─────────────┐
│  Supervisor │ ──────────────────► │  RAG Agent  │
│  :3000      │ ◄────────────────── │  :3001      │
└─────────────┘     JSON Response    └─────────────┘
       │
       │ POST /invoke
       ▼
┌─────────────┐
│Memory Agent │
│  :3002      │
└─────────────┘
```

### Flow

```
1. Supervisor sends HTTP POST to http://rag-agent:3001/invoke
   {
     "messages": [{"role": "user", "content": "search auth docs"}]
   }

2. RAG Agent processes and returns HTTP response
   {
     "messages": [...],
     "output": "Found 3 documents about authentication..."
   }
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Simple to implement | Tight coupling (need to know URLs) |
| Easy to debug (curl, Postman) | No built-in load balancing |
| No extra infrastructure | No built-in retry/resilience |
| Low latency | Sync blocking calls |
| Universal - works everywhere | Need service discovery in K8s |

### When to Use

- Simple setups
- Direct request/response needed
- Low latency requirements
- Debugging/development
- Small number of agents

---

## Pattern 3: gRPC (Protocol Buffers)

### Architecture

```
┌─────────────────┐    gRPC/Protobuf    ┌─────────────────┐
│   Supervisor    │ ◄─────────────────► │   RAG Agent     │
│   (Client)      │    Bidirectional    │   gRPC Server   │
│                 │    Streaming        │   :50053        │
└─────────────────┘                     └─────────────────┘
         │
         │ gRPC
         ▼
┌─────────────────┐                     ┌─────────────────┐
│  zerg-tasks     │                     │  zerg-vector    │
│  :50051         │                     │  :50052         │
└─────────────────┘                     └─────────────────┘
```

### Proto Definition

```protobuf
// manifests/grpc/proto/apps/v1/agents.proto
syntax = "proto3";
package agents.v1;

service AgentService {
  // Unary: Simple invoke
  rpc Invoke(InvokeRequest) returns (InvokeResponse);

  // Server streaming: Stream response tokens
  rpc Stream(InvokeRequest) returns (stream StreamChunk);

  // Bidirectional: Multi-turn conversation
  rpc Converse(stream ConversationMessage) returns (stream ConversationMessage);
}

message InvokeRequest {
  string agent_name = 1;
  repeated Message messages = 2;
  map<string, string> config = 3;
}

message InvokeResponse {
  repeated Message messages = 1;
  map<string, string> metadata = 2;
}

message StreamChunk {
  string content = 1;
  bool done = 2;
}

message Message {
  string role = 1;    // "user", "assistant", "system"
  string content = 2;
}
```

### Flow

```
1. Supervisor creates gRPC client
   const client = new AgentServiceClient("rag-agent:50053");

2. Invoke agent (unary)
   const response = await client.invoke({
     agentName: "rag",
     messages: [{ role: "user", content: "search auth docs" }]
   });

3. Or stream response
   const stream = client.stream({ agentName: "rag", messages: [...] });
   for await (const chunk of stream) {
     console.log(chunk.content);
   }
```

### TypeScript Client (Connect Protocol)

```typescript
// libs/rpc-ts/src/agents.ts
import { createClient } from "@connectrpc/connect";
import { AgentService } from "./gen/agents/v1/agents_connect";

export function createAgentClient(baseUrl: string) {
  return createClient(AgentService, createConnectTransport({ baseUrl }));
}

// Usage in supervisor
const ragClient = createAgentClient("http://rag-agent:50053");
const response = await ragClient.invoke({
  agentName: "rag",
  messages: [{ role: "user", content: "search docs" }]
});
```

### Rust Agent Server

```rust
// If agent is implemented in Rust
use rpc::agents::agent_service_server::{AgentService, AgentServiceServer};

#[tonic::async_trait]
impl AgentService for MyAgent {
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let req = request.into_inner();
        // Process with LLM...
        Ok(Response::new(InvokeResponse { ... }))
    }

    type StreamStream = ReceiverStream<Result<StreamChunk, Status>>;

    async fn stream(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<Self::StreamStream>, Status> {
        // Stream tokens as they're generated
    }
}
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Strong typing (protobuf contracts) | Need to define .proto files |
| Excellent performance (binary, HTTP/2) | More setup than HTTP |
| Bidirectional streaming | Browser needs Connect/gRPC-Web |
| Already have infrastructure (tasks, vector) | LangChain doesn't have native gRPC |
| Works great with Rust services | Extra codegen step |
| Compression built-in (Zstd) | |
| Service reflection for debugging | |

### When to Use

- Agents need to call Rust services (Tasks, Vector)
- Need strong API contracts between agents
- High-throughput agent invocations
- Want bidirectional streaming (multi-turn)
- Polyglot environment (Rust + TypeScript)
- Already using gRPC in the stack

### Integration with Existing Services

```
┌─────────────┐      ┌──────────────────────────────────┐
│   UI/Web    │─────►│          zerg-api (:8080)        │
└─────────────┘ HTTP │            REST Gateway           │
                     └──────────────┬───────────────────┘
                                    │ gRPC
                     ┌──────────────┴───────────────────┐
                     │        zerg-grpc (:50051)        │
                     │  Tasks + Vector + AgentService   │
                     └──────────────┬───────────────────┘
                                    │ gRPC (internal)
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
        ┌──────────┐          ┌──────────┐          ┌──────────┐
        │RAG Agent │          │ Memory   │          │ Supervisor│
        │  :50053  │          │  :50054  │          │  :50055   │
        └──────────┘          └──────────┘          └──────────┘
```

---

## Pattern 4: RemoteRunnable (LangServe)

### Architecture

```
┌─────────────────┐    RemoteRunnable     ┌─────────────────┐
│   Supervisor    │ ◄─────────────────► │   LangServe     │
│   (LangGraph)   │    HTTP + Streaming   │   RAG Agent     │
│                 │                       │   :8000/rag     │
└─────────────────┘                       └─────────────────┘
         │
         │ RemoteRunnable
         ▼
┌─────────────────┐
│   LangServe     │
│   Memory Agent  │
│   :8001/memory  │
└─────────────────┘
```

### Flow

```
1. Supervisor creates RemoteRunnable
   const ragAgent = new RemoteRunnable({
     url: "http://rag-agent:8000/rag"
   });

2. Invoke like a local graph
   const result = await ragAgent.invoke({
     messages: [...]
   });

3. Supports streaming out of the box
   for await (const chunk of ragAgent.stream({...})) {
     console.log(chunk);
   }
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Native LangGraph integration | LangGraph/LangChain specific |
| Streaming support built-in | Requires LangServe deployment |
| Same API as local graphs | Limited to LangGraph ecosystem |
| State checkpointing support | HTTP-based (same limitations) |
| Tracing integration (LangSmith) | |

### When to Use

- All agents are LangGraph
- Need streaming responses
- Want consistent API (local = remote)
- Using LangSmith for observability
- LangGraph Platform deployment

---

## Pattern 5: A2A (Agent2Agent Protocol)

### Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    A2A Protocol (JSON-RPC 2.0)                  │
└────────────────────────────────────────────────────────────────┘
         │                    │                    │
    Agent Card           Agent Card           Agent Card
    Discovery            Discovery            Discovery
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │Supervisor│         │RAG Agent│         │Memory   │
    │A2A Client│         │A2A Server│        │A2A Server│
    │         │         │:4001    │         │:4002    │
    └─────────┘         └─────────┘         └─────────┘
         │
         │ 1. GET /.well-known/agent-card.json
         │ 2. POST /a2a (JSON-RPC)
         ▼
```

### Agent Card Example

```json
{
  "name": "rag-agent",
  "description": "RAG retrieval agent for document search",
  "version": "1.0.0",
  "url": "http://rag-agent:4001/a2a",
  "capabilities": {
    "streaming": true,
    "pushNotifications": false
  },
  "skills": [
    {
      "id": "document-search",
      "name": "Document Search",
      "description": "Search indexed documents"
    }
  ]
}
```

### Flow

```
1. Supervisor discovers agent via Agent Card
   GET http://rag-agent:4001/.well-known/agent-card.json

2. Send task via JSON-RPC
   POST http://rag-agent:4001/a2a
   {
     "jsonrpc": "2.0",
     "method": "tasks/send",
     "params": {
       "message": {
         "role": "user",
         "parts": [{"text": "search auth docs"}]
       }
     },
     "id": "req-123"
   }

3. Receive response (or stream via SSE)
   {
     "jsonrpc": "2.0",
     "result": {
       "id": "task-456",
       "status": "completed",
       "artifacts": [...]
     },
     "id": "req-123"
   }
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Industry standard (150+ partners) | Newer, evolving spec |
| Framework agnostic | More setup than HTTP |
| Built-in discovery (Agent Cards) | Slightly more complex |
| Streaming support (SSE) | Need to implement server |
| Long-running task support | |
| Security model built-in | |

### When to Use

- Cross-organization agents
- Multi-vendor environments
- Need standardized discovery
- Long-running tasks
- Public/marketplace agents
- Future-proofing architecture

---

## Comparison Matrix

| Feature | HTTP | gRPC | NATS | RemoteRunnable | A2A |
|---------|------|------|------|----------------|-----|
| **Complexity** | Low | Medium | Medium | Low | Medium |
| **Coupling** | Tight | Medium | Loose | Medium | Loose |
| **Discovery** | Manual | Manual | Manual | Manual | Built-in |
| **Streaming** | Manual | Bidirectional | Yes | Yes | Yes |
| **Load Balancing** | External | External | Built-in | External | External |
| **Resilience** | Poor | Good | Excellent | Poor | Good |
| **Standards** | REST | Protobuf | NATS protocol | LangServe | A2A spec |
| **Multi-language** | Yes | Yes | Yes | Python/TS | Yes |
| **Latency** | ~1-2ms | ~0.5-1ms | ~1-5ms | ~1-2ms | ~1-2ms |
| **Type Safety** | None/Zod | Strong (proto) | Schema-based | TypeScript | JSON Schema |
| **Rust Integration** | Fair | Excellent | Good | None | Fair |

---

## Implementation Files

```
apps/agents/
├── agent-patterns/           # Examples of all patterns
│   ├── src/
│   │   ├── patterns/
│   │   │   ├── http/
│   │   │   │   ├── server.ts      # HTTP agent server
│   │   │   │   └── client.ts      # HTTP supervisor client
│   │   │   │
│   │   │   ├── grpc/              # NEW: gRPC pattern
│   │   │   │   ├── server.ts      # gRPC agent server (Connect)
│   │   │   │   ├── client.ts      # gRPC client
│   │   │   │   └── README.md      # Setup instructions
│   │   │   │
│   │   │   ├── nats/
│   │   │   │   ├── server.ts      # NATS agent server
│   │   │   │   ├── client.ts      # NATS supervisor client
│   │   │   │   └── types.ts       # Message types
│   │   │   │
│   │   │   ├── remote-runnable/
│   │   │   │   ├── server.ts      # LangServe server
│   │   │   │   └── client.ts      # RemoteRunnable client
│   │   │   │
│   │   │   └── a2a/
│   │   │       ├── server.ts      # A2A agent server
│   │   │       ├── client.ts      # A2A supervisor client
│   │   │       └── agent-card.ts  # Agent card definition
│   │   │
│   │   └── test-all.ts           # Test all patterns
│   │
│   └── package.json

manifests/grpc/proto/apps/v1/
├── tasks.proto                # Existing: Tasks service
├── vector.proto               # Existing: Vector service
└── agents.proto               # NEW: Agent service definition

libs/rpc-ts/src/
└── agents/                    # NEW: Generated TS client for agents
```

---

## Quick Decision Guide

```
                         Start
                           │
                           ▼
                 ┌─────────────────┐
                 │ Need cross-org  │──Yes──► A2A Protocol
                 │ interoperability?│
                 └────────┬────────┘
                          │ No
                          ▼
                 ┌─────────────────┐
                 │ Need async/     │──Yes──► NATS
                 │ high resilience?│
                 └────────┬────────┘
                          │ No
                          ▼
                 ┌─────────────────┐
                 │ Need strong     │──Yes──► gRPC
                 │ typing / Rust   │
                 │ integration?    │
                 └────────┬────────┘
                          │ No
                          ▼
                 ┌─────────────────┐
                 │ All agents are  │──Yes──► RemoteRunnable
                 │ LangGraph?      │
                 └────────┬────────┘
                          │ No
                          ▼
                        HTTP
```

### Decision Matrix by Use Case

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Agents calling Rust services | **gRPC** | Already have infra, strong typing |
| Simple dev/debug setup | **HTTP** | Easiest to test with curl |
| Production with scaling | **NATS** | Resilience, load balancing |
| LangGraph-only stack | **RemoteRunnable** | Native streaming, same API |
| Public/marketplace agents | **A2A** | Industry standard discovery |
| High-throughput streaming | **gRPC** | Bidirectional, binary protocol |
| Fire-and-forget tasks | **NATS** | Async, retry built-in |

---

## Running the Examples

```bash
# Install dependencies
cd apps/agents/agent-patterns
bun install

# Start infrastructure
docker compose -f manifests/dockers/compose.yaml up nats -d

# Test individual patterns
bun run test:http          # HTTP pattern
bun run test:grpc          # gRPC pattern (requires proto codegen)
bun run test:nats          # NATS pattern
bun run test:remote        # RemoteRunnable pattern
bun run test:a2a           # A2A pattern

# Test all patterns
bun run test:all

# For gRPC, ensure proto is compiled first
cd ../../../
nx run rpc-ts:build        # Generate TypeScript clients
```

---

## Next Steps

1. Choose pattern based on requirements (see Decision Guide above)
2. For gRPC: Create `agents.proto` and run codegen
3. Implement servers for each agent
4. Update supervisor to use chosen pattern
5. Deploy to Kubernetes
6. Add observability (OpenTelemetry / Braintrust)

## Recommendation for This Repo

Given the architecture:
- **AI Agents**: TypeScript (LangGraph/LangChain)
- **gRPC Servers**: Rust (Tonic) - `zerg-tasks`, `zerg-vector`
- **gRPC Clients**: TypeScript (Connect) + Rust (Tonic in zerg-api)
- **Frontends**: TanStack, Astro - can use gRPC or REST
- **Proxy**: `zerg-api` - REST-to-gRPC for simple clients

### Recommended Pattern by Use Case

| Use Case | Pattern | Why |
|----------|---------|-----|
| **TS Agent → Rust Service** | gRPC (Connect) | Direct, typed, streaming |
| **Frontend → Backend** | REST via zerg-api | Simpler auth, caching |
| **Agent → Agent** | HTTP or NATS | Start simple, scale with NATS |
| **Long-running tasks** | NATS | Resilience, retry built-in |

### Implementation Priority

1. **gRPC (Connect)** - For TS agents calling Rust services directly
   - Create `agents.proto` for agent-specific operations
   - Generate TS clients with `@connectrpc/connect`
   - Agents get typed access to Tasks, Vector services

2. **REST via zerg-api** - For web frontends
   - Keep REST for browser-based apps (simpler)
   - zerg-api proxies to gRPC internally

3. **NATS** - For agent-to-agent async workflows
   - When agents need to communicate without blocking
   - Fire-and-forget patterns

4. **HTTP** - For development/debugging
   - Easy to test with curl
   - Good for initial agent development

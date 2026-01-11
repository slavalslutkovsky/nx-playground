# API Request Patterns

This document describes the 4 core patterns for handling requests from UI clients through the Rust Axum API gateway.

## Architecture Overview

**zerg-api** is a **REST-to-gRPC proxy**. It serves two purposes:
1. **REST API** for web clients that prefer HTTP/JSON
2. **gRPC proxy** - translates REST calls to internal gRPC services

**Clients have two options:**
- **Direct gRPC** (Connect protocol) - TypeScript agents, fullstack apps (TanStack/Astro)
- **REST via zerg-api** - Simple clients, browsers without gRPC support

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENTS                                         │
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │  TanStack    │    │    Astro     │    │  AI Agents   │                 │
│   │  (React)     │    │              │    │ (TypeScript) │                 │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                 │
│          │                   │                   │                          │
│    ┌─────┴─────┐       ┌─────┴─────┐       ┌─────┴─────┐                   │
│    │gRPC│ REST │       │gRPC│ REST │       │gRPC│ REST │                   │
│    └──┬───┬────┘       └──┬───┬────┘       └──┬───┬────┘                   │
│       │   │               │   │               │   │                         │
└───────┼───┼───────────────┼───┼───────────────┼───┼─────────────────────────┘
        │   │               │   │               │   │
        │   └───────────────┼───┴───────────────┼───┘
        │                   │                   │
        │ Direct gRPC       │      REST         │ Direct gRPC
        │ (Connect)         ▼                   │ (Connect)
        │           ┌───────────────┐           │
        │           │   zerg-api    │           │
        │           │ REST-to-gRPC  │           │
        │           │    Proxy      │           │
        │           │   :8080       │           │
        │           └───────┬───────┘           │
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │ gRPC (internal)
                            ▼
              ┌─────────────────────────────┐
              │      gRPC Servers (Rust)    │
              │  zerg-tasks    zerg-vector  │
              │   :50051         :50052     │
              └─────────────────────────────┘
```

## Request Flow Options

| Client Type | Recommended | Why |
|-------------|-------------|-----|
| **TS Agents** | Direct gRPC (Connect) | Typed, streaming, no proxy hop |
| **TanStack/Astro** | Either | gRPC for perf, REST for simplicity |
| **Browser (no build)** | REST via zerg-api | No gRPC setup needed |
| **Mobile** | REST via zerg-api | Simpler integration |

---

## Pattern Summary

```
UI Clients (Web/Mobile/Agents)
        │
        │ HTTP/JSON or gRPC
        ▼
┌───────────────────────────────────────────────────────────────────────┐
│                    zerg-api (Axum Gateway :8080)                       │
│                                                                        │
│  ┌──────────────┬──────────────┬──────────────┬──────────────────┐   │
│  │   Pattern 1  │   Pattern 2  │   Pattern 3  │     Pattern 4    │   │
│  │  Direct DB   │    gRPC      │  Async Msg   │    AI Agents     │   │
│  └──────┬───────┴──────┬───────┴──────┬───────┴────────┬─────────┘   │
└─────────┼──────────────┼──────────────┼────────────────┼─────────────┘
          │              │              │                │
          ▼              ▼              ▼                ▼
    ┌──────────┐  ┌────────────┐  ┌──────────┐   ┌──────────────┐
    │PostgreSQL│  │gRPC Service│  │NATS/Redis│   │Agent Gateway │
    │          │  │Tasks/Vector│  │  Stream  │   │  Supervisor  │
    └──────────┘  └────────────┘  └──────────┘   └──────────────┘
```

| Pattern | Transport | Response | Use Case | Latency |
|---------|-----------|----------|----------|---------|
| **1. Direct DB** | SQL | Sync | Simple CRUD, auth, sessions | ~1-5ms |
| **2. gRPC Service** | gRPC/Protobuf | Sync | Domain operations, streaming | ~2-10ms |
| **3. Async Messaging** | NATS/Redis | Async | Background jobs, notifications | ~5-50ms |
| **4. AI Agents** | HTTP/gRPC | Sync/Stream | AI workflows, RAG, tools | ~100ms-30s |

---

## Pattern 1: Direct Database Access

Handler communicates directly with PostgreSQL. Best for simple operations that don't require domain logic separation.

### Flow

```
┌────────┐      ┌──────────┐      ┌────────────┐
│ Client │ ───► │ zerg-api │ ───► │ PostgreSQL │
│        │ ◄─── │  (Axum)  │ ◄─── │            │
└────────┘      └──────────┘      └────────────┘
      HTTP/JSON      │        SQL
                     ▼
              ┌──────────┐
              │  Redis   │ (sessions)
              └──────────┘
```

### When to Use

- User authentication and sessions
- Simple CRUD without complex business logic
- Configuration/settings endpoints
- Health checks with DB verification
- Operations where domain service is overkill

### Implementation

```rust
// apps/zerg/api/src/handlers/users.rs
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(user))
}
```

### Files

- `apps/zerg/api/src/handlers/` - Direct DB handlers
- `libs/database/` - Connection pooling, migrations
- `libs/domains/users/` - User domain with direct DB

---

## Pattern 2: gRPC to Domain Service

Handler calls a dedicated gRPC microservice for domain operations. Best for complex business logic that benefits from separation.

### Flow

```
┌────────┐      ┌──────────┐  gRPC   ┌─────────────┐      ┌────────────┐
│ Client │ ───► │ zerg-api │ ──────► │ zerg-tasks  │ ───► │ PostgreSQL │
│        │ ◄─── │  (Axum)  │ ◄────── │   :50051    │ ◄─── │            │
└────────┘      └──────────┘         └─────────────┘      └────────────┘
      HTTP/JSON         gRPC/Protobuf           SQL

                        OR

┌────────┐      ┌──────────┐  gRPC   ┌─────────────┐      ┌────────┐
│ Client │ ───► │ zerg-api │ ──────► │ zerg-vector │ ───► │ Qdrant │
│        │ ◄─── │  (Axum)  │ ◄────── │   :50052    │ ◄─── │        │
└────────┘      └──────────┘         └─────────────┘      └────────┘
```

### When to Use

- Domain operations with business logic
- Need strong typing via protobuf contracts
- Streaming data (large result sets)
- Operations that may be called by multiple services
- Team ownership boundaries

### Implementation

```rust
// apps/zerg/api/src/handlers/tasks.rs
pub async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<CreateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    // Call gRPC service
    let response = state
        .tasks_client
        .create(tonic::Request::new(proto::CreateRequest {
            title: input.title,
            description: input.description,
            priority: input.priority.into(),
        }))
        .await?
        .into_inner();

    Ok(Json(response.task.into()))
}

// Streaming example
pub async fn list_tasks_stream(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stream = state
        .tasks_client
        .list_stream(proto::ListStreamRequest {})
        .await?
        .into_inner();

    // Convert to SSE stream for client
    Sse::new(stream.map(|item| Event::default().json_data(item)))
}
```

### Proto Definition

```protobuf
// manifests/grpc/proto/apps/v1/tasks.proto
service TasksService {
  rpc Create(CreateRequest) returns (CreateResponse);
  rpc GetById(GetByIdRequest) returns (GetByIdResponse);
  rpc List(ListRequest) returns (ListResponse);
  rpc ListStream(ListStreamRequest) returns (stream ListStreamResponse);
  rpc UpdateById(UpdateByIdRequest) returns (UpdateByIdResponse);
  rpc DeleteById(DeleteByIdRequest) returns (DeleteByIdResponse);
}
```

### Files

- `manifests/grpc/proto/apps/v1/` - Proto definitions
- `apps/zerg/tasks/` - Tasks gRPC service
- `apps/zerg/vector/` - Vector gRPC service
- `libs/rpc/` - Rust gRPC codegen
- `libs/domains/tasks/` - Task domain logic

---

## Pattern 3: Async Messaging (NATS/Redis Streams)

Handler publishes a job to a message queue, returns immediately. Worker processes asynchronously.

### Flow

```
┌────────┐      ┌──────────┐  Publish  ┌─────────────┐
│ Client │ ───► │ zerg-api │ ────────► │ NATS Stream │
│        │ ◄─── │  (Axum)  │           │ EMAIL_JOBS  │
└────────┘      └──────────┘           └──────┬──────┘
      HTTP/JSON    │                          │
           202 Accepted                       │ Subscribe
                   │                          ▼
                   │               ┌─────────────────┐
                   │               │ zerg-email-nats │
                   │               │    (Worker)     │
                   │               └────────┬────────┘
                   │                        │
                   │                        ▼
                   │               ┌─────────────────┐
                   │               │ SendGrid / SES  │
                   │               └─────────────────┘
                   │
                   ▼ (optional webhook/polling)
            ┌────────────┐
            │ Job Status │
            │  Endpoint  │
            └────────────┘
```

### When to Use

- Fire-and-forget operations (emails, notifications)
- Long-running tasks (report generation, exports)
- Operations that can fail and retry
- Fan-out to multiple consumers
- Decoupling producer from consumer

### Implementation

```rust
// apps/zerg/api/src/handlers/email.rs
pub async fn send_email(
    State(state): State<AppState>,
    Json(input): Json<SendEmailRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let job = EmailJob {
        id: Uuid::new_v4(),
        to: input.to,
        subject: input.subject,
        body: input.body,
        retry_count: 0,
    };

    // Publish to NATS
    state.nats_client
        .publish("EMAIL_JOBS", serde_json::to_vec(&job)?)
        .await?;

    // Return job ID for tracking
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "job_id": job.id, "status": "queued" }))
    ))
}

// Optional: Job status endpoint
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatus>, ApiError> {
    // Check Redis or DB for job status
    let status = state.redis.get::<_, JobStatus>(&format!("job:{}", job_id)).await?;
    Ok(Json(status))
}
```

### Worker Implementation

```rust
// apps/zerg/email-nats/src/lib.rs
impl Processor for EmailProcessor {
    type Job = EmailJob;

    async fn process(&self, job: EmailJob) -> Result<(), ProcessError> {
        match &self.provider {
            EmailProvider::SendGrid(client) => client.send(&job).await,
            EmailProvider::Smtp(client) => client.send(&job).await,
        }

        // Update job status in Redis
        self.redis.set(&format!("job:{}", job.id), "completed").await?;
        Ok(())
    }
}
```

### Files

- `libs/core/messaging/` - Job/Processor traits
- `libs/core/nats-worker/` - NATS JetStream framework
- `libs/core/stream-worker/` - Redis Streams alternative
- `apps/zerg/email-nats/` - Email worker

---

## Pattern 4: AI Agent Invocation

Handler invokes AI agents for intelligent workflows. Can be sync (wait for result) or streaming (SSE).

### Flow

```
┌────────┐      ┌──────────┐  HTTP   ┌─────────────────┐
│ Client │ ───► │ zerg-api │ ──────► │  Agent Gateway  │
│        │ ◄─── │  (Axum)  │ ◄────── │     :8080       │
└────────┘      └──────────┘         └────────┬────────┘
      HTTP/JSON       │                       │
      or SSE          │               ┌───────┴───────┐
                      │               ▼               ▼
                      │        ┌────────────┐  ┌────────────┐
                      │        │ Supervisor │  │ RAG Agent  │
                      │        │  (Claude)  │  │ (Claude)   │
                      │        └─────┬──────┘  └────────────┘
                      │              │
                      │     ┌────────┼────────┐
                      │     ▼        ▼        ▼
                      │  ┌─────┐  ┌─────┐  ┌─────┐
                      │  │ RAG │  │Tools│  │Mem  │
                      │  │Agent│  │Agent│  │Agent│
                      │  └─────┘  └─────┘  └─────┘
```

### When to Use

- Natural language queries
- Document search (RAG)
- Multi-step reasoning
- Tool use (calendar, search, calculations)
- Context-aware operations

### Implementation

```rust
// apps/zerg/api/src/handlers/agents.rs
pub async fn invoke_agent(
    State(state): State<AppState>,
    Path(agent_name): Path<String>,
    Json(input): Json<AgentRequest>,
) -> Result<Json<AgentResponse>, ApiError> {
    let response = state.http_client
        .post(format!("{}/agents/{}/invoke", state.agent_gateway_url, agent_name))
        .json(&input)
        .send()
        .await?
        .json::<AgentResponse>()
        .await?;

    Ok(Json(response))
}

// Streaming version (SSE)
pub async fn stream_agent(
    State(state): State<AppState>,
    Path(agent_name): Path<String>,
    Json(input): Json<AgentRequest>,
) -> impl IntoResponse {
    let response = state.http_client
        .post(format!("{}/agents/{}/stream", state.agent_gateway_url, agent_name))
        .json(&input)
        .send()
        .await?;

    // Forward SSE stream to client
    Sse::new(response.bytes_stream().map(|chunk| {
        Event::default().data(String::from_utf8_lossy(&chunk?))
    }))
}
```

### Agent Communication Patterns

The agents themselves can communicate using 4 sub-patterns (see `docs/plans/agent-communication-patterns.md`):

| Pattern | Transport | Best For |
|---------|-----------|----------|
| NATS | Message Queue | Distributed, resilient |
| HTTP | REST/JSON | Simple, debugging |
| RemoteRunnable | LangServe | LangGraph-native |
| A2A Protocol | JSON-RPC 2.0 | Cross-vendor interop |

### Files

- `apps/agents/gateway/` - Agent gateway (Express)
- `apps/agents/supervisor-langgraph/` - Multi-agent supervisor
- `apps/agents/rag-agent/` - RAG agent
- `libs/agents/` - Shared agent libraries

---

## Testing Strategy

### Testing Pyramid

```
         ┌───────────────────┐
         │    E2E Tests      │  ~10 tests, 15-30s
         │  (Full System)    │  UI → API → Services → DB
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │  Handler Tests    │  ~30 tests, 2-5s per domain
         │  (HTTP Layer)     │  HTTP request → response
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │ Integration Tests │  ~100 tests, 10s
         │ (Service + DB)    │  Service → Repository → DB
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │    Unit Tests     │  ~300 tests, instant
         │ (Business Logic)  │  Pure functions, mocks
         └───────────────────┘
```

---

### Pattern 1 Testing: Direct DB

#### Unit Test (Mock DB)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        UserRepo {}
        #[async_trait]
        impl UserRepository for UserRepo {
            async fn get_by_id(&self, id: Uuid) -> Result<Option<User>>;
        }
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_| Ok(None));

        let result = get_user_service(&mock_repo, Uuid::new_v4()).await;
        assert!(matches!(result, Err(UserError::NotFound)));
    }
}
```

#### Integration Test (Real DB)

```rust
// libs/domains/users/tests/integration_test.rs
use test_utils::TestDatabase;

#[tokio::test]
async fn test_create_and_get_user() {
    let db = TestDatabase::new().await;
    let repo = PgUserRepository::new(db.pool());

    let user = repo.create(CreateUser {
        email: "test@example.com".into(),
        name: "Test User".into(),
    }).await.unwrap();

    let found = repo.get_by_id(user.id).await.unwrap();
    assert_eq!(found.unwrap().email, "test@example.com");
}
```

#### Handler Test (HTTP)

```rust
// libs/domains/users/tests/handler_test.rs
use axum_test::TestServer;

#[tokio::test]
async fn test_get_user_endpoint() {
    let db = TestDatabase::new().await;
    let app = create_app(db.pool());
    let server = TestServer::new(app).unwrap();

    // Create user first
    let create_resp = server
        .post("/api/users")
        .json(&json!({ "email": "test@example.com", "name": "Test" }))
        .await;
    assert_eq!(create_resp.status_code(), 201);

    let user: User = create_resp.json();

    // Get user
    let get_resp = server.get(&format!("/api/users/{}", user.id)).await;
    assert_eq!(get_resp.status_code(), 200);
}
```

---

### Pattern 2 Testing: gRPC Service

#### Unit Test (Mock gRPC Client)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        TasksClient {}
        #[async_trait]
        impl TasksServiceClient for TasksClient {
            async fn create(&self, req: CreateRequest) -> Result<CreateResponse>;
        }
    }

    #[tokio::test]
    async fn test_create_task_via_grpc() {
        let mut mock_client = MockTasksClient::new();
        mock_client
            .expect_create()
            .returning(|req| Ok(CreateResponse {
                task: Some(Task {
                    id: Uuid::new_v4().as_bytes().to_vec(),
                    title: req.title,
                    ..Default::default()
                }),
            }));

        let result = create_task_handler(&mock_client, CreateTaskRequest {
            title: "Test Task".into(),
            description: None,
        }).await;

        assert!(result.is_ok());
    }
}
```

#### Integration Test (Real gRPC Server)

```rust
// apps/zerg/tasks/tests/integration_test.rs
use tonic::transport::Channel;
use testcontainers::clients::Cli;

#[tokio::test]
async fn test_tasks_grpc_integration() {
    let docker = Cli::default();
    let db = TestDatabase::new(&docker).await;

    // Start gRPC server in background
    let addr = start_grpc_server(db.pool()).await;

    // Create client
    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = TasksServiceClient::new(channel);

    // Test create
    let response = client
        .create(CreateRequest {
            title: "Integration Test Task".into(),
            description: "Testing gRPC".into(),
            priority: Priority::High as i32,
        })
        .await
        .unwrap();

    assert!(response.into_inner().task.is_some());
}
```

#### Contract Test (Proto Compatibility)

```rust
// Ensure proto changes don't break compatibility
#[test]
fn test_proto_backwards_compatibility() {
    // Old message format
    let old_bytes = hex::decode("0a0474657374").unwrap();

    // Should still deserialize with new proto
    let task: Task = prost::Message::decode(&old_bytes[..]).unwrap();
    assert_eq!(task.title, "test");
}
```

---

### Pattern 3 Testing: Async Messaging

#### Unit Test (Mock Publisher)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        NatsClient {}
        #[async_trait]
        impl Publisher for NatsClient {
            async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_send_email_publishes_job() {
        let mut mock_nats = MockNatsClient::new();
        mock_nats
            .expect_publish()
            .withf(|subject, _| subject == "EMAIL_JOBS")
            .times(1)
            .returning(|_, _| Ok(()));

        let result = send_email_handler(&mock_nats, SendEmailRequest {
            to: "user@example.com".into(),
            subject: "Test".into(),
            body: "Hello".into(),
        }).await;

        assert!(result.is_ok());
    }
}
```

#### Integration Test (Real NATS)

```rust
// apps/zerg/email-nats/tests/integration_test.rs
use testcontainers::images::nats::Nats;

#[tokio::test]
async fn test_email_worker_processes_job() {
    let docker = Cli::default();
    let nats_container = docker.run(Nats::default());
    let nats_url = format!("nats://localhost:{}", nats_container.get_host_port_ipv4(4222));

    // Setup
    let nats_client = async_nats::connect(&nats_url).await.unwrap();
    let jetstream = async_nats::jetstream::new(nats_client.clone());

    // Create stream
    jetstream.create_stream(StreamConfig {
        name: "EMAIL_JOBS".into(),
        subjects: vec!["EMAIL_JOBS".into()],
        ..Default::default()
    }).await.unwrap();

    // Start worker in background
    let worker = EmailWorker::new(nats_url.clone(), MockEmailProvider::new());
    let worker_handle = tokio::spawn(async move { worker.run().await });

    // Publish job
    let job = EmailJob {
        id: Uuid::new_v4(),
        to: "test@example.com".into(),
        subject: "Test".into(),
        body: "Hello".into(),
        retry_count: 0,
    };
    jetstream.publish("EMAIL_JOBS", serde_json::to_vec(&job).unwrap().into()).await.unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify job was processed (check mock or Redis status)
    // ...

    worker_handle.abort();
}
```

#### E2E Test (Full Flow)

```rust
#[tokio::test]
async fn test_email_e2e_flow() {
    // Start all services
    let docker = Cli::default();
    let db = TestDatabase::new(&docker).await;
    let nats = docker.run(Nats::default());

    // Start API and worker
    let api = start_api(db.pool(), nats.url()).await;
    let worker = start_email_worker(nats.url()).await;

    // Send request via HTTP
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/email/send", api.url()))
        .json(&json!({
            "to": "test@example.com",
            "subject": "E2E Test",
            "body": "Hello from E2E"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
    let body: serde_json::Value = resp.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();

    // Poll for completion
    let status = poll_job_status(&client, &api.url(), job_id, Duration::from_secs(5)).await;
    assert_eq!(status, "completed");
}
```

---

### Pattern 4 Testing: AI Agents

#### Unit Test (Mock LLM)

```typescript
// apps/agents/rag-agent/src/__tests__/graph.test.ts
import { describe, it, expect, vi } from 'vitest';
import { graph } from '../retrieval_graph/graph';

describe('RAG Agent', () => {
  it('should generate query and retrieve documents', async () => {
    // Mock LLM responses
    const mockLLM = {
      invoke: vi.fn()
        .mockResolvedValueOnce({ content: 'refined search query' })
        .mockResolvedValueOnce({ content: 'Based on the documents...' })
    };

    // Mock retriever
    const mockRetriever = {
      invoke: vi.fn().mockResolvedValue([
        { pageContent: 'Document 1', metadata: {} },
        { pageContent: 'Document 2', metadata: {} },
      ])
    };

    const result = await graph.invoke(
      { messages: [{ role: 'user', content: 'search auth docs' }] },
      { configurable: { llm: mockLLM, retriever: mockRetriever } }
    );

    expect(mockRetriever.invoke).toHaveBeenCalled();
    expect(result.messages).toHaveLength(2);
  });
});
```

#### Integration Test (Real Agent, Mock LLM)

```typescript
// apps/agents/rag-agent/tests/integration.test.ts
import { describe, it, expect } from 'vitest';
import { createTestGraph } from '../src/test-utils';

describe('RAG Agent Integration', () => {
  it('should handle full retrieval flow', async () => {
    const { graph, vectorStore } = await createTestGraph({
      llmProvider: 'mock', // or 'anthropic' for real LLM
      vectorProvider: 'memory',
    });

    // Seed test documents
    await vectorStore.addDocuments([
      { pageContent: 'Authentication uses JWT tokens', metadata: { source: 'auth.md' } },
      { pageContent: 'OAuth2 flow for third-party', metadata: { source: 'oauth.md' } },
    ]);

    const result = await graph.invoke({
      messages: [{ role: 'user', content: 'How does auth work?' }],
    });

    expect(result.messages.at(-1).content).toContain('JWT');
  });
});
```

#### E2E Test (Agent Gateway)

```typescript
// apps/agents/gateway/tests/e2e.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { spawn, ChildProcess } from 'child_process';

describe('Agent Gateway E2E', () => {
  let gateway: ChildProcess;
  let ragAgent: ChildProcess;

  beforeAll(async () => {
    // Start services
    ragAgent = spawn('bun', ['run', 'start'], { cwd: 'apps/agents/rag-agent' });
    gateway = spawn('bun', ['run', 'start'], { cwd: 'apps/agents/gateway' });

    // Wait for startup
    await waitForHealthy('http://localhost:8080/health');
  });

  afterAll(() => {
    gateway.kill();
    ragAgent.kill();
  });

  it('should list available agents', async () => {
    const resp = await fetch('http://localhost:8080/agents');
    const agents = await resp.json();

    expect(agents).toContainEqual(
      expect.objectContaining({ name: 'rag-agent' })
    );
  });

  it('should invoke RAG agent', async () => {
    const resp = await fetch('http://localhost:8080/agents/rag-agent/invoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        messages: [{ role: 'user', content: 'test query' }],
        config: { retrieverProvider: 'memory' }
      }),
    });

    expect(resp.status).toBe(200);
    const result = await resp.json();
    expect(result.messages).toBeDefined();
  });

  it('should stream agent responses', async () => {
    const resp = await fetch('http://localhost:8080/agents/rag-agent/stream', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        messages: [{ role: 'user', content: 'test query' }],
      }),
    });

    expect(resp.headers.get('content-type')).toContain('text/event-stream');

    const reader = resp.body.getReader();
    const chunks = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(new TextDecoder().decode(value));
    }

    expect(chunks.length).toBeGreaterThan(0);
  });
});
```

---

## Running Tests

### All Tests

```bash
# Rust tests
cargo test --workspace

# TypeScript agent tests
bun test --filter=agents

# Full E2E
docker compose -f manifests/dockers/compose.yaml up -d
cargo test --workspace -- --ignored  # runs E2E tests
```

### Pattern-Specific

```bash
# Pattern 1: Direct DB
cargo test -p domain_users

# Pattern 2: gRPC
cargo test -p zerg_tasks
cargo test -p domain_tasks --test handler_test

# Pattern 3: Async Messaging
cargo test -p zerg_email_nats

# Pattern 4: AI Agents
cd apps/agents/rag-agent && bun test
cd apps/agents/gateway && bun test:e2e
```

### Test Infrastructure

```bash
# Start test dependencies
docker compose -f manifests/dockers/compose.yaml up postgres redis nats qdrant -d

# Run with coverage
cargo llvm-cov --workspace --html
```

---

## Decision Guide

```
Start
  │
  ▼
┌─────────────────────────────┐
│ Need immediate response?     │
└──────────────┬──────────────┘
               │
       ┌───────┴───────┐
       │ Yes           │ No
       ▼               ▼
┌─────────────┐  ┌─────────────────┐
│Need complex │  │ Async Pattern 3 │
│domain logic?│  │ (NATS/Redis)    │
└──────┬──────┘  └─────────────────┘
       │
   ┌───┴───┐
   │ Yes   │ No
   ▼       ▼
┌────────┐ ┌────────────┐
│Pattern │ │ Pattern 1  │
│2: gRPC │ │ Direct DB  │
└────────┘ └────────────┘

        OR

┌─────────────────────────────┐
│ Need AI/reasoning?          │
└──────────────┬──────────────┘
               │ Yes
               ▼
        ┌─────────────┐
        │ Pattern 4   │
        │ AI Agents   │
        └─────────────┘
```

---

## Related Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Full system architecture
- [messaging-patterns.md](./messaging-patterns.md) - When to use gRPC vs NATS vs Kafka
- [plans/agent-communication-patterns.md](./plans/agent-communication-patterns.md) - 4 AI agent patterns
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - Comprehensive testing strategies
- [ADR-0001](./ADR/0001-use-grpc-for-service-communication.md) - gRPC decision
- [ADR-0002](./ADR/0002-nats-jetstream-for-messaging.md) - NATS decision

# Architecture Overview

This document provides a comprehensive overview of the nx-playground architecture, covering service design, communication patterns, Kubernetes deployment, and code organization.

## Table of Contents

- [High-Level Architecture](#high-level-architecture)
- [Service Architecture](#service-architecture)
- [Communication Patterns](#communication-patterns)
- [AI Agents Architecture](#ai-agents-architecture)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Code Organization](#code-organization)
- [Domain-Driven Design](#domain-driven-design)
- [Testing Strategy](#testing-strategy)
- [Observability](#observability)

---

## High-Level Architecture

```mermaid
flowchart TB
    subgraph Clients["Client Layer"]
        Web["Web Apps<br/>(SolidJS/React)"]
        Mobile["Mobile/External"]
    end

    subgraph Gateway["API Gateway Layer"]
        API["zerg-api<br/>(Axum REST)"]
    end

    subgraph Services["Microservices Layer"]
        Tasks["zerg-tasks<br/>(gRPC :50051)"]
        Vector["zerg-vector<br/>(gRPC :50052)"]
        Email["zerg-email-nats<br/>(NATS Worker)"]
    end

    subgraph AIAgents["AI Agents Layer"]
        Supervisor["Supervisor<br/>(LangGraph)"]
        RAG["RAG Agent"]
        Memory["Memory Agent"]
        ToolsAgent["Tools Agent"]
    end

    subgraph Messaging["Messaging Layer"]
        NATS["NATS JetStream"]
    end

    subgraph Data["Data Layer"]
        PG[(PostgreSQL)]
        Redis[(Redis)]
        Qdrant[(Qdrant<br/>Vector DB)]
        VectorStores["Vector Stores<br/>(Elastic/Pinecone/MongoDB)"]
    end

    Web -->|HTTP/gRPC-Web| API
    Mobile -->|HTTP| API

    API -->|gRPC| Tasks
    API -->|gRPC| Vector
    API -->|Publish| NATS

    Supervisor --> RAG
    Supervisor --> Memory
    Supervisor --> ToolsAgent
    ToolsAgent -->|gRPC| Tasks
    RAG --> VectorStores

    NATS -->|Subscribe| Email

    Tasks --> PG
    API --> PG
    API --> Redis
    Vector --> Qdrant
    Email -->|SendGrid/SES| External["Email Providers"]
```

---

## Service Architecture

### Services Overview

| Service | Type | Port | Technology | Purpose |
|---------|------|------|------------|---------|
| **zerg-api** | REST Gateway | 8080 | Axum + Rust | API orchestration, auth, routing |
| **zerg-tasks** | gRPC Service | 50051 | Tonic + Rust | Task domain operations |
| **zerg-vector** | gRPC Service | 50052 | Tonic + Rust | Vector embeddings & search |
| **zerg-email-nats** | Worker | 8081 | NATS + Rust | Async email processing |
| **terran-web** | Frontend | 3000 | SolidStart | Main web application |

### API Gateway Pattern

```mermaid
flowchart LR
    subgraph API["zerg-api (Gateway)"]
        Auth["JWT Auth<br/>Middleware"]
        Routes["Route<br/>Handler"]
        GrpcPool["gRPC Client<br/>Pool (Lazy)"]
    end

    Client -->|"POST /api/tasks"| Auth
    Auth -->|Validated| Routes
    Routes -->|"TasksService::Create"| GrpcPool
    GrpcPool -->|gRPC| TasksService["zerg-tasks<br/>:50051"]

    Routes -->|Direct| PG[(PostgreSQL)]
    Routes -->|Session| Redis[(Redis)]
```

**Key Features:**
- **Lazy gRPC connections**: Services don't need to be up at API startup
- **JWT + Redis sessions**: Stateless auth with session storage
- **OpenAPI/Swagger**: Auto-generated documentation at `/docs`
- **Health endpoints**: `/health` (liveness), `/ready` (readiness with DB check)

---

## Communication Patterns

### gRPC Services

```mermaid
sequenceDiagram
    participant C as Client
    participant A as zerg-api
    participant T as zerg-tasks
    participant DB as PostgreSQL

    C->>A: POST /api/tasks
    A->>A: Validate JWT
    A->>T: gRPC Create(TaskRequest)
    T->>DB: INSERT INTO tasks
    DB-->>T: Task row
    T-->>A: TaskResponse
    A-->>C: JSON Response
```

**Proto Definitions** (`manifests/grpc/proto/apps/v1/`):

```protobuf
// tasks.proto
service TasksService {
  rpc Create(CreateRequest) returns (CreateResponse);
  rpc GetById(GetByIdRequest) returns (GetByIdResponse);
  rpc List(ListRequest) returns (ListResponse);
  rpc ListStream(ListStreamRequest) returns (stream ListStreamResponse);
  rpc DeleteById(DeleteByIdRequest) returns (DeleteByIdResponse);
  rpc UpdateById(UpdateByIdRequest) returns (UpdateByIdResponse);
}

// vector.proto
service VectorService {
  rpc CreateCollection(CreateCollectionRequest) returns (CreateCollectionResponse);
  rpc UpsertWithEmbedding(UpsertWithEmbeddingRequest) returns (UpsertResponse);
  rpc SearchWithEmbedding(SearchWithEmbeddingRequest) returns (SearchResponse);
  rpc GetRecommendations(RecommendationsRequest) returns (RecommendationsResponse);
}
```

### NATS JetStream Messaging

```mermaid
flowchart LR
    subgraph Producers
        API["zerg-api"]
        Worker["Other Services"]
    end

    subgraph NATS["NATS JetStream"]
        Stream["EMAIL_JOBS<br/>(Durable Stream)"]
        DLQ["EMAIL_DLQ<br/>(Dead Letter Queue)"]
    end

    subgraph Consumers
        Email1["email-worker-1"]
        Email2["email-worker-2"]
    end

    API -->|"Publish EmailJob"| Stream
    Worker -->|"Publish EmailJob"| Stream
    Stream -->|"Pull Consumer"| Email1
    Stream -->|"Pull Consumer"| Email2
    Email1 -->|"After 3 retries"| DLQ
    Email2 -->|"After 3 retries"| DLQ
```

**NATS Worker Features:**
- **Pull-based consumers**: Backpressure handling
- **Durable subscriptions**: Survive restarts
- **Dead Letter Queue**: Failed messages after max retries
- **Graceful shutdown**: Drain in-flight messages
- **Health endpoints**: K8s probe ready

**Email Job Processing:**

```rust
// libs/core/messaging/src/lib.rs
#[async_trait]
pub trait Processor: Send + Sync {
    type Job: Job;
    async fn process(&self, job: Self::Job) -> Result<(), ProcessError>;
}

// apps/zerg/email-nats/src/lib.rs
impl Processor for EmailProcessor {
    type Job = EmailJob;

    async fn process(&self, job: EmailJob) -> Result<(), ProcessError> {
        match &self.provider {
            EmailProvider::SendGrid(client) => client.send(&job).await,
            EmailProvider::Smtp(client) => client.send(&job).await,
        }
    }
}
```

### Message Flow Decision Guide

```mermaid
flowchart TD
    Start["Need to communicate<br/>between services?"]

    Start --> Immediate{"Need immediate<br/>response?"}

    Immediate -->|Yes| gRPC["Use gRPC"]
    Immediate -->|No| Replay{"Need message<br/>replay?"}

    Replay -->|Yes| Kafka["Consider Kafka"]
    Replay -->|No| Complex{"Complex routing<br/>needed?"}

    Complex -->|Yes| RabbitMQ["Consider RabbitMQ"]
    Complex -->|No| NATS["Use NATS JetStream"]

    gRPC --> Done["Done"]
    Kafka --> Done
    RabbitMQ --> Done
    NATS --> Done
```

---

## AI Agents Architecture

The platform includes a comprehensive AI agents system built with LangChain/LangGraph, supporting multiple orchestration patterns and communication protocols.

### Agent System Overview

```mermaid
flowchart TB
    subgraph Clients["Client Layer"]
        User["User/Application"]
        LangSmith["LangSmith<br/>(Tracing)"]
    end

    subgraph Orchestration["Orchestration Layer"]
        Supervisor["Supervisor Agent<br/>(Claude Sonnet)"]
    end

    subgraph Agents["Specialized Agents"]
        RAG["RAG Agent<br/>(Document Search)"]
        Memory["Memory Agent<br/>(User Context)"]
        Tools["Tools Agent<br/>(Utilities)"]
    end

    subgraph VectorStores["Vector Stores"]
        Elastic["Elasticsearch"]
        MongoDB["MongoDB Atlas"]
        Pinecone["Pinecone"]
    end

    subgraph Backend["Backend Services"]
        TasksGRPC["zerg-tasks<br/>(gRPC :50051)"]
        VectorGRPC["zerg-vector<br/>(gRPC :50052)"]
    end

    User -->|Query| Supervisor
    Supervisor -->|Route| RAG
    Supervisor -->|Route| Memory
    Supervisor -->|Route| Tools

    RAG --> Elastic
    RAG --> MongoDB
    RAG --> Pinecone

    Tools -->|gRPC| TasksGRPC
    Tools -->|gRPC| VectorGRPC

    Supervisor --> LangSmith
    RAG --> LangSmith
```

### Agent Applications

| Agent | Location | Purpose | LLM | Key Features |
|-------|----------|---------|-----|--------------|
| **RAG Agent** | `apps/agents/rag-agent` | Document Q&A | Claude 3.5 Sonnet | Multi-vector store, query refinement |
| **Memory Agent** | `apps/agents/code-tester` | User context storage | Claude 3.5 Sonnet | LangGraph store, memory recall |
| **Tools Agent** | `apps/agents/whatsup-agent` | Utility functions | Claude Haiku | Calculator, time, weather, search |
| **Supervisor** | `apps/agents/supervisor-langgraph` | Multi-agent orchestration | Claude Sonnet 4 | Routing, delegation, composition |
| **ADK Supervisor** | `apps/agents/supervisor-adk` | Google ADK orchestration | Gemini 2.0 Flash | Native ADK tools, DevTools |

### Supervisor Pattern (Multi-Agent Orchestration)

```mermaid
sequenceDiagram
    participant U as User
    participant S as Supervisor
    participant R as RAG Agent
    participant M as Memory Agent
    participant T as Tools Agent

    U->>S: "Find auth docs and remember I'm working on OAuth"

    S->>S: Analyze request (Claude Sonnet)
    Note over S: Route to RAG Agent

    S->>R: Search for authentication docs
    R->>R: generateQuery → retrieve → respond
    R-->>S: Auth documentation results

    S->>S: Analyze next step
    Note over S: Route to Memory Agent

    S->>M: Store "working on OAuth"
    M->>M: callModel → storeMemory
    M-->>S: Memory stored confirmation

    S->>S: Analyze next step
    Note over S: FINISH - Task complete

    S-->>U: Consolidated response with docs + confirmation
```

### RAG Agent Architecture

```mermaid
flowchart LR
    subgraph Input
        Query["User Query"]
        Config["Configuration<br/>- userId<br/>- embeddingModel<br/>- retrieverProvider"]
    end

    subgraph RAGGraph["RAG Graph"]
        GenQuery["generateQuery<br/>(Claude Haiku)"]
        Retrieve["retrieve<br/>(Vector Search)"]
        Respond["respond<br/>(Claude Sonnet)"]
    end

    subgraph VectorStores["Vector Store Options"]
        ES["Elasticsearch<br/>(Cloud/Local)"]
        Mongo["MongoDB Atlas<br/>(Vector Search)"]
        Pine["Pinecone<br/>(Serverless)"]
    end

    subgraph Embeddings["Embedding Providers"]
        OpenAI["OpenAI<br/>text-embedding-3-*"]
        Cohere["Cohere<br/>embed-*"]
    end

    Query --> GenQuery
    Config --> GenQuery
    GenQuery -->|Refined Query| Retrieve
    Retrieve --> ES
    Retrieve --> Mongo
    Retrieve --> Pine
    ES --> Respond
    Mongo --> Respond
    Pine --> Respond
    Embeddings --> Retrieve
    Respond --> Output["Response with Context"]
```

**Vector Store Configuration:**

```typescript
// Configuration schema
interface RAGConfig {
  userId: string;                    // User isolation
  embeddingModel: string;            // "openai/text-embedding-3-small"
  retrieverProvider: 'elastic' | 'pinecone' | 'mongodb' | 'elastic-local';
  searchKwargs: { k: number };       // Top-k results
  responseSystemPromptTemplate: string;
  querySystemPromptTemplate: string;
}
```

### Memory Agent (LangGraph Store)

```mermaid
flowchart TB
    subgraph State["Agent State"]
        Messages["Conversation<br/>Messages"]
        Memories["Recent Memories<br/>(limit: 10)"]
    end

    subgraph Graph["Memory Graph"]
        CallModel["callModel<br/>(Inject memories)"]
        Decision{{"Tool calls?"}}
        StoreMemory["storeMemory<br/>(Persist to store)"]
        End["END"]
    end

    subgraph Store["LangGraph Store"]
        Namespace["namespace: memories/{userId}"]
    end

    Messages --> CallModel
    Memories --> CallModel
    CallModel --> Decision
    Decision -->|Yes| StoreMemory
    Decision -->|No| End
    StoreMemory --> Store
    StoreMemory --> CallModel
```

### Agent Communication Patterns

```mermaid
flowchart TB
    subgraph Patterns["Communication Protocols"]
        A2A["A2A Protocol<br/>(Agent-to-Agent)"]
        GRPC["gRPC/Connect<br/>(Service Tools)"]
        NATS["NATS Pub/Sub<br/>(Distributed)"]
        Remote["RemoteRunnable<br/>(LangServe-style)"]
    end

    subgraph A2ADetails["A2A Protocol v0.3"]
        Card["Agent Card<br/>/.well-known/agent-card.json"]
        Tasks["Task Lifecycle<br/>submitted → working → completed"]
        JSONRPC["JSON-RPC 2.0<br/>POST /a2a"]
    end

    subgraph GRPCDetails["gRPC Tools"]
        TaskOps["Task Operations<br/>create, get, list, update, delete"]
        Stream["Server Streaming<br/>ListStream"]
    end

    subgraph NATSDetails["NATS Messaging"]
        Subject["agents.{name}.request"]
        Queue["Queue Groups<br/>Load Balancing"]
        Reply["Request/Reply Pattern"]
    end

    A2A --> A2ADetails
    GRPC --> GRPCDetails
    NATS --> NATSDetails
```

### Agent Tools (gRPC Integration)

```typescript
// libs/agents/tools/src/tasks.ts
const taskTools = createTaskTools({
  serviceUrl: 'http://zerg-tasks:50051'
});

// Available tools:
// - create_task(title, description, priority)
// - get_task(taskId)
// - list_tasks(limit, offset, status)
// - update_task(taskId, updates)
// - delete_task(taskId)
```

### Shared Agent Libraries

```
libs/agents/
├── core/                    # Base agent framework
│   ├── BaseDeployableAgent  # Abstract base class
│   ├── LLM provider config  # Vertex AI, Bedrock, Anthropic, OpenAI
│   └── Service discovery    # K8s, Agent Engine, Local
│
├── tools/                   # Shared tool definitions
│   ├── Task service tools   # gRPC-backed CRUD
│   └── Utility functions    # UUID conversion, formatting
│
└── deploy/                  # Deployment utilities
    └── CLI for agent deployment
```

### LLM Provider Support

```mermaid
flowchart LR
    subgraph Agent["Agent Application"]
        Config["LLM Config<br/>'provider/model'"]
    end

    subgraph Providers["Supported Providers"]
        Anthropic["anthropic<br/>Claude 3.5/4"]
        OpenAI["openai<br/>GPT-4o"]
        Vertex["vertex-ai<br/>Gemini"]
        Bedrock["bedrock<br/>Claude/Titan"]
        LiteLLM["litellm<br/>Abstraction"]
    end

    Config --> Anthropic
    Config --> OpenAI
    Config --> Vertex
    Config --> Bedrock
    Config --> LiteLLM
```

**Configuration Examples:**

```typescript
// Provider/model format
const llm = await initChatModel('anthropic/claude-sonnet-4-20250514');
const llm = await initChatModel('openai/gpt-4o');
const llm = await initChatModel('vertex-ai/gemini-2.0-flash');
```

### Agent Deployment (GKE)

```yaml
# apps/agents/deployable/task-agent/deploy/gke/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-agent
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: task-agent
          image: task-agent:latest
          env:
            - name: K8S_DEPLOYMENT
              value: "true"
            - name: NODE_ENV
              value: "production"
          resources:
            requests:
              cpu: "100m"
              memory: "256Mi"
            limits:
              cpu: "500m"
              memory: "512Mi"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: kubernetes.io/hostname
```

### Service Discovery by Environment

| Environment | Task Service | User Service | Discovery Method |
|-------------|--------------|--------------|------------------|
| **Local** | `localhost:50051` | `localhost:50052` | Env vars |
| **GKE** | `task-service.default.svc.cluster.local:8080` | `user-service.default.svc.cluster.local:8080` | K8s DNS |
| **Agent Engine** | `task-service.internal:8080` | `user-service.internal:8080` | VPC internal |

---

## Kubernetes Deployment

### Cluster Architecture

```mermaid
flowchart TB
    subgraph Internet
        Users["Users"]
    end

    subgraph K8s["Kubernetes Cluster"]
        subgraph Ingress["Ingress Layer"]
            Nginx["nginx-ingress"]
        end

        subgraph Apps["zerg namespace"]
            WebDeploy["zerg-web<br/>Deployment"]
            APIDeploy["zerg-api<br/>Deployment"]
            TasksDeploy["zerg-tasks<br/>Deployment"]
            VectorDeploy["zerg-vector<br/>Deployment"]
            EmailDeploy["zerg-email-nats<br/>Deployment"]
        end

        subgraph DBs["dbs namespace"]
            PG["PostgreSQL"]
            Redis["Redis"]
            NATS["NATS"]
            Qdrant["Qdrant"]
        end

        subgraph Secrets["Secret Management"]
            Vault["HashiCorp Vault"]
            ESO["External Secrets<br/>Operator"]
        end

        subgraph Monitoring["Observability"]
            Prometheus["Prometheus"]
            Grafana["Grafana"]
        end
    end

    Users --> Nginx
    Nginx --> WebDeploy
    Nginx --> APIDeploy

    APIDeploy --> TasksDeploy
    APIDeploy --> VectorDeploy
    APIDeploy --> PG
    APIDeploy --> Redis
    APIDeploy --> NATS

    TasksDeploy --> PG
    VectorDeploy --> Qdrant
    EmailDeploy --> NATS

    ESO --> Vault
    ESO --> APIDeploy
    ESO --> EmailDeploy

    Prometheus --> APIDeploy
    Prometheus --> TasksDeploy
    Prometheus --> EmailDeploy
```

### Kustomize Structure

```
k8s/
├── core/
│   ├── base/
│   │   ├── namespace.yaml          # zerg namespace
│   │   └── configmaps/
│   │       └── shared-config.yaml  # DB pooling, service URLs
│   └── overlays/
│       ├── dev/
│       │   ├── ingress.yaml        # zerg.local, api.zerg.local
│       │   └── kustomization.yaml
│       └── prod/
│           ├── ingress.yaml        # TLS with cert-manager
│           └── kustomization.yaml
│
├── external-secrets/
│   ├── base/
│   │   ├── vault-secret-store.yaml
│   │   ├── gcp-secret-store.yaml
│   │   └── zerg-secrets.yaml       # DB, auth, OAuth secrets
│   └── overlays/
│       └── prod/
│
├── observability/
│   ├── base/
│   │   └── service-monitors/
│   └── overlays/
│       └── dev/
│           └── prometheus-helm.yaml
│
└── gitops/
    └── base/
        └── flux-sync.yaml          # FluxCD configuration
```

### Service Deployment Pattern

```mermaid
flowchart TB
    subgraph Deployment["zerg-api Deployment"]
        subgraph Pod1["Pod 1"]
            Container1["zerg-api<br/>container"]
        end
        subgraph Pod2["Pod 2"]
            Container2["zerg-api<br/>container"]
        end
    end

    subgraph Config["Configuration"]
        CM["ConfigMap<br/>shared-config"]
        Secret["ExternalSecret<br/>→ Vault"]
    end

    subgraph Scaling["Auto Scaling"]
        HPA["HPA<br/>CPU: 70%<br/>Memory: 80%<br/>2-10 replicas"]
    end

    subgraph Service["Service"]
        SVC["ClusterIP<br/>:8080"]
    end

    CM --> Pod1
    CM --> Pod2
    Secret --> Pod1
    Secret --> Pod2
    HPA --> Deployment
    Pod1 --> SVC
    Pod2 --> SVC
```

### Deployment Configuration

```yaml
# apps/zerg/api/k8s/kustomize/base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: zerg-api
spec:
  replicas: 2
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: zerg-api
          resources:
            requests:
              memory: "128Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              # No CPU limit - prevents throttling
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            periodSeconds: 10
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            periodSeconds: 5
            failureThreshold: 3
```

### Secret Management Flow

```mermaid
flowchart LR
    subgraph Vault["HashiCorp Vault"]
        KV["KV v2 Engine"]
        AWS["AWS Secrets Engine"]
    end

    subgraph K8s["Kubernetes"]
        ESO["External Secrets<br/>Operator"]
        ES["ExternalSecret<br/>CRD"]
        Secret["K8s Secret"]
        Pod["Application Pod"]
    end

    KV -->|"secret/data/zerg/*"| ESO
    AWS -->|"aws/creds/ses-sender"| ESO
    ESO --> ES
    ES -->|"Sync every 1h"| Secret
    Secret -->|"env vars"| Pod
```

**Secrets Managed:**
- `DATABASE_USER`, `DATABASE_PASSWORD` (Vault KV)
- `JWT_SECRET` (Vault KV)
- `GOOGLE_CLIENT_ID/SECRET`, `GITHUB_CLIENT_ID/SECRET` (OAuth)
- `AWS_ACCESS_KEY_ID/SECRET` (Dynamic - 30min refresh)
- `SENDGRID_API_KEY` (Email)

---

## Code Organization

### Monorepo Structure

```
nx-playground/
├── apps/
│   ├── zerg/                    # Rust microservices
│   │   ├── api/                 # REST API gateway
│   │   ├── tasks/               # Tasks gRPC service
│   │   ├── vector/              # Vector gRPC service
│   │   ├── email-nats/          # NATS email worker
│   │   └── web/                 # Legacy web frontend
│   │
│   ├── terran/                  # Frontend applications
│   │   ├── web/                 # SolidJS + SolidStart
│   │   ├── internal-tools/      # React admin dashboard
│   │   └── astro/               # Static site
│   │
│   └── agents/                  # AI agents
│       ├── code-tester/         # LangChain code evaluator
│       ├── rag-agent/           # RAG with embeddings
│       └── supervisor-*/        # Agent orchestrators
│
├── libs/
│   ├── core/                    # Shared Rust utilities
│   │   ├── axum-helpers/        # HTTP middleware, JWT
│   │   ├── config/              # Environment config
│   │   ├── grpc/                # gRPC client utilities
│   │   ├── messaging/           # Job/Processor traits
│   │   ├── nats-worker/         # NATS JetStream framework
│   │   └── proc_macros/         # Code generation macros
│   │
│   ├── domains/                 # Business domains
│   │   ├── tasks/               # Task management
│   │   ├── projects/            # Project management
│   │   ├── users/               # User auth & profiles
│   │   ├── vector/              # Vector operations
│   │   └── cloud_resources/     # Cloud infrastructure
│   │
│   ├── database/                # DB connectors
│   ├── migration/               # SeaORM migrations
│   ├── rpc/                     # Rust proto codegen
│   ├── rpc-ts/                  # TypeScript proto codegen
│   │
│   └── agents/                  # Agent libraries
│       ├── core/                # LangChain base
│       ├── tools/               # Shared agent tools
│       └── deploy/              # Deployment CLI
│
├── manifests/
│   ├── grpc/proto/              # Proto definitions
│   └── dockers/                 # Docker compose
│
├── k8s/                         # Kubernetes manifests
│   ├── core/                    # Namespace, ConfigMaps
│   ├── external-secrets/        # Vault integration
│   ├── observability/           # Prometheus, Grafana
│   └── gitops/                  # FluxCD
│
└── docs/                        # Documentation
```

### Dependency Flow

```mermaid
flowchart TB
    subgraph Apps["Applications"]
        API["zerg-api"]
        Tasks["zerg-tasks"]
        Vector["zerg-vector"]
        Email["zerg-email-nats"]
    end

    subgraph Domains["Domain Libraries"]
        DomainTasks["domain_tasks"]
        DomainProjects["domain_projects"]
        DomainUsers["domain_users"]
        DomainVector["domain_vector"]
    end

    subgraph Core["Core Libraries"]
        AxumHelpers["axum-helpers"]
        Config["config"]
        Messaging["messaging"]
        NatsWorker["nats-worker"]
        Grpc["grpc"]
    end

    subgraph Data["Data Layer"]
        Database["database"]
        Migration["migration"]
        RPC["rpc"]
    end

    API --> DomainProjects
    API --> DomainUsers
    API --> AxumHelpers
    API --> Grpc

    Tasks --> DomainTasks
    Tasks --> Database

    Vector --> DomainVector

    Email --> Messaging
    Email --> NatsWorker

    DomainTasks --> Database
    DomainProjects --> Database
    DomainUsers --> Database

    Database --> Migration
```

---

## Domain-Driven Design

### 4-Layer Architecture

```mermaid
flowchart TB
    subgraph Layer4["Layer 4: Handlers"]
        Handlers["HTTP/gRPC Handlers<br/>- Route definitions<br/>- Request/Response mapping<br/>- Status codes"]
    end

    subgraph Layer3["Layer 3: Service"]
        Service["Business Logic<br/>- Validation<br/>- Business rules<br/>- Orchestration"]
    end

    subgraph Layer2["Layer 2: Repository"]
        Repo["Data Access<br/>- CRUD operations<br/>- Query building<br/>- Transaction management"]
    end

    subgraph Layer1["Layer 1: Models"]
        Models["Data Structures<br/>- Entities<br/>- DTOs<br/>- Enums"]
    end

    Handlers --> Service
    Service --> Repo
    Repo --> Models
```

### Domain Structure

```
libs/domains/projects/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── models.rs        # Project, CreateProject, UpdateProject
│   ├── error.rs         # ProjectError enum
│   ├── repository.rs    # ProjectRepository trait
│   ├── postgres.rs      # PgProjectRepository impl
│   ├── service.rs       # ProjectService with business logic
│   └── handlers.rs      # Axum HTTP handlers
│
└── tests/
    ├── integration_test.rs  # DB integration tests
    └── handler_test.rs      # HTTP handler tests
```

### Repository Pattern

```rust
// Trait definition (repository.rs)
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, input: CreateProject) -> Result<Project>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Project>>;
    async fn list(&self, filter: ProjectFilter) -> Result<Vec<Project>>;
    async fn update(&self, id: Uuid, input: UpdateProject) -> Result<Project>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

// PostgreSQL implementation (postgres.rs)
pub struct PgProjectRepository {
    pool: PgPool,
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, input: CreateProject) -> Result<Project> {
        sqlx::query_as(/* SQL */)
            .bind(/* params */)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }
}
```

---

## Testing Strategy

### Testing Pyramid

```mermaid
flowchart TB
    subgraph E2E["E2E Tests (~10)"]
        E2EDesc["Full app + auth + all domains<br/>apps/zerg/api/tests/e2e_test.rs<br/>~15-30s"]
    end

    subgraph Handler["Handler Tests (~30)"]
        HandlerDesc["HTTP handlers per domain<br/>libs/domains/*/tests/handler_test.rs<br/>~2-5s"]
    end

    subgraph Integration["Integration Tests (~100)"]
        IntDesc["Service + Repository + DB<br/>libs/domains/*/tests/integration_test.rs<br/>~10s"]
    end

    subgraph Unit["Unit Tests (~300)"]
        UnitDesc["Business logic with mocks<br/>libs/domains/*/src/**/tests<br/>~0.00s"]
    end

    E2E --> Handler --> Integration --> Unit
```

### Test Infrastructure

```rust
// libs/testing/test-utils/src/lib.rs

// Auto-managed PostgreSQL container
pub struct TestDatabase {
    container: ContainerAsync<Postgres>,
    connection: DatabaseConnection,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let container = Postgres::default().start().await;
        // Auto-runs migrations
        // Auto-cleanup on drop
    }
}

// Deterministic test data
pub struct TestDataBuilder {
    seed: u64,
}

impl TestDataBuilder {
    pub fn from_test_name(name: &str) -> Self {
        Self { seed: hash(name) }
    }

    pub fn user_id(&self) -> Uuid {
        // Deterministic UUID from seed
    }

    pub fn name(&self, prefix: &str, suffix: &str) -> String {
        format!("{}-{}-{}", prefix, self.seed, suffix)
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Unit tests only (fast)
cargo test --lib --workspace

# Integration tests (requires Docker)
cargo test --workspace --test integration_test

# Handler tests
cargo test --workspace --test handler_test

# E2E tests
cargo test -p zerg_api --test e2e_test

# Specific domain
cargo test -p domain_projects
```

---

## Observability

### Metrics & Monitoring

```mermaid
flowchart LR
    subgraph Apps["Applications"]
        API["zerg-api<br/>/metrics"]
        Tasks["zerg-tasks<br/>/metrics"]
        Email["zerg-email-nats<br/>/metrics"]
    end

    subgraph Monitoring["Monitoring Stack"]
        Prometheus["Prometheus"]
        Grafana["Grafana"]
    end

    API -->|"scrape 30s"| Prometheus
    Tasks -->|"scrape 30s"| Prometheus
    Email -->|"scrape 30s"| Prometheus
    Prometheus --> Grafana
```

### Health Check Pattern

```rust
// libs/core/nats-worker/src/health.rs
pub struct HealthServer {
    port: u16,
    ready: Arc<AtomicBool>,
}

impl HealthServer {
    // /health - Liveness (app is running)
    async fn health(&self) -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "service": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    // /ready - Readiness (app can serve traffic)
    async fn ready(&self) -> impl IntoResponse {
        if self.ready.load(Ordering::SeqCst) {
            (StatusCode::OK, "ready")
        } else {
            (StatusCode::SERVICE_UNAVAILABLE, "not ready")
        }
    }

    // /metrics - Prometheus metrics
    async fn metrics(&self) -> impl IntoResponse {
        // Prometheus format metrics
    }
}
```

### Tracing Configuration

```rust
// libs/core/config/src/lib.rs
pub fn setup_tracing(environment: Environment) {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env());

    match environment {
        Environment::Production => {
            // JSON structured logs for log aggregation
            subscriber.json().init();
        }
        Environment::Development => {
            // Pretty colored output for local dev
            subscriber.pretty().init();
        }
    }
}
```

---

## Quick Reference

### Ports

| Service | Port | Protocol |
|---------|------|----------|
| zerg-api | 8080 | HTTP |
| zerg-tasks | 50051 | gRPC |
| zerg-vector | 50052 | gRPC |
| zerg-email-nats | 8081 | HTTP (metrics) |
| PostgreSQL | 5432 | TCP |
| Redis | 6379 | TCP |
| NATS | 4222 | TCP |
| Qdrant | 6333 | HTTP/gRPC |

### Environment Variables

```bash
# Database
DATABASE_URL=postgres://user:pass@localhost/db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2

# Redis
REDIS_URL=redis://localhost:6379

# NATS
NATS_URL=nats://localhost:4222

# gRPC Services
TASKS_SERVICE_ADDR=http://[::1]:50051
VECTOR_SERVICE_ADDR=http://[::1]:50052

# Auth
JWT_SECRET=your-32-char-minimum-secret
GOOGLE_CLIENT_ID=...
GITHUB_CLIENT_ID=...

# Email
EMAIL_PROVIDER=sendgrid  # or smtp
SENDGRID_API_KEY=...
```

### Common Commands

```bash
# Start infrastructure
docker compose -f manifests/dockers/compose.yaml up -d

# Run migrations
just _migration

# Build all
cargo build --workspace

# Run API
cargo run -p zerg_api

# Run Tasks service
cargo run -p zerg_tasks

# Run tests
cargo test --workspace

# Format & lint
cargo fmt && cargo clippy

# K8s deploy (dev)
kubectl apply -k k8s/core/overlays/dev
```

---

## Related Documentation

- [Modular Monolith Architecture](./modular-monolith-architecture.md) - Domain design patterns
- [Messaging Patterns](./messaging-patterns.md) - When to use gRPC vs NATS vs Kafka
- [gRPC Guide](./grpc.md) - Streaming patterns and best practices
- [Testing Guide](./TESTING_GUIDE.md) - Comprehensive testing strategies
- [Code Reuse Patterns](./code-reuse-patterns.md) - Reducing boilerplate across domains

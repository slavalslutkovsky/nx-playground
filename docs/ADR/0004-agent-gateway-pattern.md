# ADR-0004: Unified Agent Gateway

## Status

Accepted

## Date

2024-04-01

## Context

With multiple AI agents deployed, we face challenges:
- Each agent exposes different endpoints
- No unified authentication
- No centralized rate limiting
- Difficult to monitor agent health
- No service discovery for agents

## Decision

Implement a **Unified Agent Gateway** that provides a single entry point for all agent interactions.

### Implementation

Location: `apps/agents/gateway`

### Architecture

```
Client
  │
  ▼
┌─────────────────────────────────────┐
│         Agent Gateway               │
│                                     │
│  ┌─────────────────────────────┐   │
│  │   Authentication (JWT/API)   │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │   Rate Limiting (per key)   │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │   Tracing (Braintrust)      │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │   Agent Registry & Router   │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
  │
  ├──► RAG Agent
  ├──► Memory Agent
  ├──► Tools Agent
  └──► Supervisor
```

### API Design

```
GET  /agents              # List agents
GET  /agents/{name}       # Agent details
POST /agents/{name}/invoke # Invoke agent
POST /agents/{name}/stream # SSE streaming
GET  /agents/{name}/health # Health check
GET  /.well-known/agent-card.json # A2A discovery
```

## Consequences

### Positive

- **Single Entry Point**: One URL for all agents
- **Unified Auth**: Consistent authentication across agents
- **Centralized Rate Limiting**: Protect LLM costs
- **Health Monitoring**: Aggregate agent health
- **Tracing**: All requests traced to Braintrust
- **A2A Compatible**: Supports agent discovery protocol

### Negative

- **Single Point of Failure**: Gateway down = all agents down
  - Mitigation: Multiple replicas, health checks
- **Added Latency**: Extra hop for all requests
  - Mitigation: Keep gateway lightweight
- **Complexity**: Another service to maintain

### Risks

- **Bottleneck**: High traffic could overwhelm gateway
  - Mitigation: HPA scaling, KEDA for event-driven scaling
- **Stale Registry**: Agent health info could be outdated
  - Mitigation: Background health checks, cache TTL

## Alternatives Considered

### Direct Agent Access

- Each agent has public endpoint
- Rejected: No unified auth, monitoring, rate limiting

### Service Mesh (Istio)

- Let mesh handle routing/auth
- Rejected: Overkill, doesn't handle agent-specific needs

### API Gateway (Kong/Ambassador)

- General purpose API gateway
- Rejected: Need agent-specific features (streaming, registry)

## References

- [A2A Protocol](https://github.com/agent-to-agent/a2a-protocol)
- [apps/agents/gateway](../../apps/agents/gateway)
- [docs/AGENTS.md](../AGENTS.md)

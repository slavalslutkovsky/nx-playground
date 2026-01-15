# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the nx-playground platform.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences.

## Template

Use the template in [0000-template.md](./0000-template.md) when creating new ADRs.

## Index

| Number | Title | Status | Date |
|--------|-------|--------|------|
| [0001](./0001-use-grpc-for-service-communication.md) | Use gRPC for Service Communication | Accepted | 2024-01-15 |
| [0002](./0002-nats-jetstream-for-messaging.md) | NATS JetStream for Async Messaging | Accepted | 2024-02-01 |
| [0003](./0003-langgraph-for-agents.md) | LangGraph for AI Agent Orchestration | Accepted | 2024-03-01 |
| [0004](./0004-agent-gateway-pattern.md) | Unified Agent Gateway | Accepted | 2024-04-01 |
| [0005](./0005-braintrust-for-agent-tracing.md) | Braintrust for Agent Tracing | Accepted | 2024-04-15 |

## Creating a New ADR

```bash
# Copy template
cp docs/ADR/0000-template.md docs/ADR/00XX-title-of-decision.md

# Edit the new file
# Update this README to add the new ADR to the index
```

## ADR Lifecycle

1. **Proposed** - Under discussion
2. **Accepted** - Decision made and implemented
3. **Deprecated** - No longer recommended
4. **Superseded** - Replaced by another ADR

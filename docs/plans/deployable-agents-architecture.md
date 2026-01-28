# Deployable Agents Architecture

This document describes the architecture for agents that can be deployed to **both GKE and Vertex AI Agent Engine** using the same codebase.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Deployable Agents                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Shared Agent Library                          │   │
│  │   libs/agents/                                                   │   │
│  │   ├── core/          # Base agent interfaces & environment      │   │
│  │   ├── tools/         # Reusable tools (gRPC, NATS, HTTP)        │   │
│  │   └── deploy/        # Deployment utilities & CLI               │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                           │                                             │
│          ┌────────────────┴────────────────┐                           │
│          ▼                                 ▼                            │
│  ┌───────────────────┐          ┌────────────────────┐                 │
│  │   GKE Deployment  │          │   Agent Engine     │                 │
│  │   (Production)    │          │   (Client/Proto)   │                 │
│  ├───────────────────┤          ├────────────────────┤                 │
│  │ • Full infra      │          │ • Fast deploy      │                 │
│  │ • NATS/gRPC       │          │ • Per-client       │                 │
│  │ • Custom scaling  │          │ • Isolated         │                 │
│  │ • Multi-provider  │          │ • Managed scaling  │                 │
│  └───────────────────┘          └────────────────────┘                 │
│          │                                 │                            │
│          └────────────────┬────────────────┘                           │
│                           ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Rust gRPC Services                            │   │
│  │   (Shared backend - accessible from both deployments)           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
libs/agents/
├── core/                     # Core interfaces and base classes
│   └── src/
│       ├── types.ts          # Type definitions
│       ├── base-agent.ts     # BaseDeployableAgent class
│       ├── environment.ts    # Environment detection
│       └── deployment.ts     # Manifest generation
│
├── tools/                    # Reusable agent tools
│   └── src/
│       ├── grpc-tools.ts     # Generic gRPC tool factory
│       ├── task-tools.ts     # Task service tools
│       └── utils.ts          # Tool utilities
│
└── deploy/                   # Deployment utilities
    └── src/
        ├── gke.ts            # GKE deployment
        ├── agent-engine.ts   # Agent Engine deployment
        └── cli.ts            # CLI tool

apps/agents/deployable/
└── task-agent/               # Example deployable agent
    ├── src/
    │   ├── agent.ts          # Agent definition
    │   ├── server.ts         # HTTP server
    │   └── test.ts           # Tests
    └── deploy/
        ├── gke/              # K8s manifests
        └── agent-engine/     # Python deploy script
```

## Creating a Deployable Agent

### 1. Extend BaseDeployableAgent

```typescript
import { BaseDeployableAgent, type AgentMetadata, type DeploymentConfig } from '@org/agents-core';
import { createTaskTools } from '@org/agents-tools';

export class TaskAgent extends BaseDeployableAgent {
  metadata: AgentMetadata = {
    name: 'task-agent',
    version: '1.0.0',
    description: 'Manages tasks via natural language',
  };

  tools = createTaskTools().map(tool => ({
    name: tool.name,
    description: tool.description,
    tool,
  }));

  deploymentConfig: DeploymentConfig = {
    gke: {
      namespace: 'agents',
      replicas: 2,
      resources: {
        requests: { cpu: '100m', memory: '256Mi' },
        limits: { cpu: '500m', memory: '512Mi' },
      },
    },
    agentEngine: {
      displayName: 'Task Manager Agent',
      description: 'AI task management agent',
      region: 'us-central1',
      requirements: ['langchain>=0.3.0', 'langgraph>=0.2.0'],
    },
  };
}
```

### 2. Create HTTP Server (for GKE)

```typescript
import express from 'express';
import { TaskAgent } from './agent.js';

const app = express();
const agent = new TaskAgent();

app.post('/invoke', async (req, res) => {
  const result = await agent.invoke({ message: req.body.message });
  res.json(result);
});

app.listen(8080);
```

### 3. Export for Agent Engine

```typescript
// For Agent Engine deployment
export function createAgent() {
  const agent = new TaskAgent();
  return agent.getRunnable();
}
```

## Deployment

### GKE Deployment

```bash
# Using NX
nx deploy:gke task-agent

# Or manually
kubectl apply -k apps/agents/deployable/task-agent/deploy/gke/
```

### Agent Engine Deployment

```bash
# Using NX
nx deploy:agent-engine task-agent

# Or manually
python apps/agents/deployable/task-agent/deploy/agent-engine/deploy.py \
  --project my-project \
  --region us-central1
```

### Using the CLI

```bash
# Deploy to GKE
agent-deploy -t gke -p my-project -c my-cluster apps/agents/deployable/task-agent

# Deploy to Agent Engine
agent-deploy -t agent-engine -p my-project apps/agents/deployable/task-agent
```

## Environment Detection

The agent automatically detects its deployment environment:

```typescript
import { env, detectDeploymentTarget } from '@org/agents-core';

// Automatic detection
console.log(env.deploymentTarget); // 'gke' | 'agent-engine' | 'local'

// Environment-specific behavior
if (env.isGKE) {
  // Use LiteLLM for provider abstraction
} else if (env.isAgentEngine) {
  // Use Vertex AI directly
}
```

## LLM Provider Selection

| Environment | Provider | Model |
|-------------|----------|-------|
| Local | Configurable | Any |
| GKE | LiteLLM (multi-provider) | vertex_ai/gemini-2.0-flash |
| Agent Engine | Vertex AI | gemini-2.0-flash |

Override per-agent:

```typescript
class MyAgent extends BaseDeployableAgent {
  getLLMConfig(target: DeploymentTarget): LLMProviderConfig {
    if (target === 'agent-engine') {
      return { provider: 'vertex-ai', model: 'gemini-2.0-flash' };
    }
    // GKE: Use Claude via Bedrock
    return { provider: 'bedrock', model: 'anthropic.claude-3-sonnet' };
  }
}
```

## Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        GCP Project                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     Shared VPC                            │  │
│  │                                                           │  │
│  │  ┌─────────────┐              ┌─────────────────────┐    │  │
│  │  │    GKE      │              │   Agent Engine      │    │  │
│  │  │  Cluster    │◀────────────▶│   (VPC Connector)   │    │  │
│  │  │             │              │                     │    │  │
│  │  │ • Agents    │              │ • Client Agents     │    │  │
│  │  │ • Rust gRPC │              │ • Prototypes        │    │  │
│  │  │ • NATS      │              │                     │    │  │
│  │  └─────────────┘              └─────────────────────┘    │  │
│  │         │                              │                  │  │
│  │         └──────────────┬───────────────┘                  │  │
│  │                        ▼                                  │  │
│  │              ┌─────────────────┐                          │  │
│  │              │  Internal LB    │                          │  │
│  │              │  (gRPC services)│                          │  │
│  │              └─────────────────┘                          │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## When to Use Each Target

| Scenario | Target | Reason |
|----------|--------|--------|
| Internal production | GKE | Full control, existing infra |
| Client-specific agent | Agent Engine | Isolation, fast deploy |
| Prototype / Demo | Agent Engine | Quick iteration |
| High-volume, cost-sensitive | GKE | Provider switching |
| Complex state management | GKE | Full state control |
| Simple RAG agent | Agent Engine | Built-in integrations |

## Testing

```bash
# Run tests for the task agent
nx test task-agent

# Test locally before deployment
nx serve task-agent
```

## Best Practices

1. **Keep agents stateless** - Store state in external services (Redis, PostgreSQL)
2. **Use environment detection** - Let the agent adapt to its deployment target
3. **Share tools via libs** - Don't duplicate tool implementations
4. **Test with mock services** - Use mock gRPC servers for unit tests
5. **Version your agents** - Include version in metadata for tracking

## Related Documentation

- [gRPC + Rust + TypeScript Architecture](./grpc-rust-typescript-agents.md)
- [Agent Communication Patterns](./agent-communication-patterns.md)

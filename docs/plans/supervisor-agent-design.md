# Supervisor Agent Design

This document outlines the architecture for a **Supervisor Agent** that orchestrates the three existing LangGraph agents as sub-agents.

> **Updated**: January 2025 - Using official `@langchain/langgraph-supervisor` package

## Overview

The Supervisor Agent acts as an intelligent router that:
1. Analyzes incoming user requests
2. Routes to the appropriate sub-agent(s)
3. Aggregates results and maintains conversation context
4. Can invoke multiple agents in sequence or parallel

## Existing Sub-Agents

| Agent | Pattern | Purpose | Location | LangGraph Version |
|-------|---------|---------|----------|-------------------|
| **RAG Agent** | Retrieval-Augmented Generation | Knowledge-grounded Q&A from documents | `apps/agents/rag-agent/` | `^0.3.0` |
| **Code-Tester** | Memory/Checkpointer | Persistent user memory management | `apps/agents/code-tester/` | `^0.3.0` |
| **Whatsup** | ReAct (Reasoning + Acting) | General tool execution | `apps/agents/whatsup-agent/` | `langchain ^1.2.3` |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      SUPERVISOR AGENT (createSupervisor)                         │
│                                                                                  │
│  ┌─────────────┐    ┌──────────────────────────────────────────────────────┐   │
│  │   START     │───▶│              SUPERVISOR LLM                          │   │
│  └─────────────┘    │                                                       │   │
│                     │  • Receives user message                              │   │
│                     │  • Decides which agent(s) to invoke via tool calls    │   │
│                     │  • Uses handoff tools for agent delegation            │   │
│                     └───────────────────────┬──────────────────────────────┘   │
│                                             │                                   │
│                           ┌─────────────────┼─────────────────┐                 │
│                           │                 │                 │                 │
│                           ▼                 ▼                 ▼                 │
│  ┌────────────────────────────┐ ┌──────────────────┐ ┌──────────────────────┐  │
│  │   transfer_to_rag_expert   │ │transfer_to_memory│ │transfer_to_tools_agent│  │
│  │        (handoff tool)      │ │   (handoff tool) │ │    (handoff tool)    │  │
│  └──────────┬─────────────────┘ └────────┬─────────┘ └──────────┬───────────┘  │
│             │                            │                      │              │
│             ▼                            ▼                      ▼              │
│  ┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐  │
│  │     RAG EXPERT       │  │    MEMORY EXPERT     │  │    TOOLS EXPERT      │  │
│  │  (createReactAgent)  │  │  (createReactAgent)  │  │  (createReactAgent)  │  │
│  │                      │  │                      │  │                      │  │
│  │ Tools:               │  │ Tools:               │  │ Tools:               │  │
│  │ • search_documents   │  │ • upsert_memory      │  │ • calculator         │  │
│  │ • query_knowledge    │  │ • recall_memory      │  │ • get_time           │  │
│  │                      │  │                      │  │ • get_weather        │  │
│  │                      │  │                      │  │ • web_search         │  │
│  └──────────┬───────────┘  └──────────┬───────────┘  └──────────┬───────────┘  │
│             │                         │                         │              │
│             │    transfer_back_to_supervisor (automatic)        │              │
│             └─────────────────────────┼─────────────────────────┘              │
│                                       ▼                                        │
│                          ┌─────────────────────────┐                           │
│                          │     SUPERVISOR LLM      │                           │
│                          │                         │                           │
│                          │ • Receives agent result │                           │
│                          │ • Decides: respond or   │                           │
│                          │   call another agent    │                           │
│                          └───────────┬─────────────┘                           │
│                                      │                                         │
│                           ┌──────────┴──────────┐                              │
│                           ▼                     ▼                              │
│                    ┌─────────────┐       ┌─────────────┐                       │
│                    │     END     │       │ Call Agent  │                       │
│                    │  (respond)  │       │   (loop)    │                       │
│                    └─────────────┘       └─────────────┘                       │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Approaches

### Approach 1: Official `@langchain/langgraph-supervisor` (RECOMMENDED)

The official package provides `createSupervisor()` which handles all the orchestration automatically.

### Approach 2: Manual StateGraph (Advanced Control)

Build custom routing logic with explicit graph nodes for maximum control.

---

## Approach 1: Official Package Implementation

### Installation

```bash
npm install @langchain/langgraph-supervisor @langchain/langgraph @langchain/core @langchain/anthropic
# or
bun add @langchain/langgraph-supervisor @langchain/langgraph @langchain/core @langchain/anthropic
```

### File Structure

```
apps/agents/supervisor-agent/
├── src/
│   ├── agents/
│   │   ├── rag-expert.ts         # RAG specialist agent
│   │   ├── memory-expert.ts      # Memory specialist agent
│   │   └── tools-expert.ts       # Tools specialist agent
│   ├── tools/
│   │   ├── rag-tools.ts          # Document search tools
│   │   ├── memory-tools.ts       # Memory CRUD tools
│   │   └── utility-tools.ts      # Calculator, time, weather
│   ├── graph.ts                  # Main supervisor graph
│   └── index.ts                  # Export
├── langgraph.json
├── package.json
└── tsconfig.json
```

### Complete Implementation

```typescript
// src/graph.ts
import { ChatAnthropic } from "@langchain/anthropic";
import { createSupervisor } from "@langchain/langgraph-supervisor";
import { createReactAgent } from "@langchain/langgraph/prebuilt";
import { tool } from "@langchain/core/tools";
import { MemorySaver, InMemoryStore } from "@langchain/langgraph";
import { z } from "zod";

// ============================================================================
// Model Configuration
// ============================================================================

const supervisorModel = new ChatAnthropic({
  model: "claude-sonnet-4-20250514",
  temperature: 0,
});

const agentModel = new ChatAnthropic({
  model: "claude-haiku-3-5-20241022",
  temperature: 0,
});

// ============================================================================
// RAG Expert Tools
// ============================================================================

const searchDocuments = tool(
  async ({ query, collection }) => {
    // In production, call your RAG agent's retrieval logic
    // or make gRPC call to zerg-vector service
    return `Found 3 documents matching "${query}" in ${collection}:\n` +
      `1. Authentication Guide - OAuth2 implementation details\n` +
      `2. API Security - Best practices for securing endpoints\n` +
      `3. User Management - Role-based access control`;
  },
  {
    name: "search_documents",
    description: "Search the knowledge base for relevant documents",
    schema: z.object({
      query: z.string().describe("The search query"),
      collection: z.string().default("default").describe("Collection to search"),
    }),
  }
);

const queryKnowledge = tool(
  async ({ question }) => {
    // RAG retrieval + generation
    return `Based on the knowledge base: The authentication system uses JWT tokens ` +
      `with a 15-minute expiry. Refresh tokens are stored in HTTP-only cookies.`;
  },
  {
    name: "query_knowledge",
    description: "Ask a question and get an answer from the knowledge base",
    schema: z.object({
      question: z.string().describe("The question to answer"),
    }),
  }
);

// ============================================================================
// Memory Expert Tools
// ============================================================================

const upsertMemory = tool(
  async ({ content, context, memoryId }, config) => {
    // In production, use LangGraph Store
    const userId = config?.configurable?.userId || "default";
    const id = memoryId || crypto.randomUUID();
    return `Memory saved for user ${userId}: [${id}] "${content}" (context: ${context})`;
  },
  {
    name: "upsert_memory",
    description: "Save or update a memory about the user",
    schema: z.object({
      content: z.string().describe("The memory content to store"),
      context: z.string().describe("Context about when/why this is relevant"),
      memoryId: z.string().optional().describe("ID to update existing memory"),
    }),
  }
);

const recallMemory = tool(
  async ({ query }, config) => {
    const userId = config?.configurable?.userId || "default";
    // In production, search the store
    return `Memories for ${userId} matching "${query}":\n` +
      `- Prefers TypeScript over JavaScript\n` +
      `- Working on authentication feature\n` +
      `- Uses VSCode with Vim keybindings`;
  },
  {
    name: "recall_memory",
    description: "Search for stored memories about the user",
    schema: z.object({
      query: z.string().describe("What to search for in memories"),
    }),
  }
);

// ============================================================================
// Tools Expert Tools (from whatsup-agent)
// ============================================================================

const calculator = tool(
  async ({ operation, a, b }) => {
    const ops: Record<string, (a: number, b: number) => number> = {
      add: (a, b) => a + b,
      subtract: (a, b) => a - b,
      multiply: (a, b) => a * b,
      divide: (a, b) => a / b,
    };
    const result = ops[operation]?.(a, b);
    return result !== undefined ? `${a} ${operation} ${b} = ${result}` : "Invalid operation";
  },
  {
    name: "calculator",
    description: "Perform arithmetic calculations",
    schema: z.object({
      operation: z.enum(["add", "subtract", "multiply", "divide"]),
      a: z.number(),
      b: z.number(),
    }),
  }
);

const getCurrentTime = tool(
  async ({ timezone }) => {
    const now = new Date();
    return `Current time in ${timezone || "UTC"}: ${now.toLocaleString("en-US", {
      timeZone: timezone || "UTC",
      dateStyle: "full",
      timeStyle: "long",
    })}`;
  },
  {
    name: "get_current_time",
    description: "Get the current date and time",
    schema: z.object({
      timezone: z.string().optional().describe("Timezone (e.g., 'America/New_York')"),
    }),
  }
);

const getWeather = tool(
  async ({ location }) => {
    // Simulated weather - replace with real API
    const conditions = ["sunny", "cloudy", "rainy", "partly cloudy"];
    const temp = Math.floor(Math.random() * 30) + 10;
    return `Weather in ${location}: ${conditions[Math.floor(Math.random() * 4)]}, ${temp}°C`;
  },
  {
    name: "get_weather",
    description: "Get current weather for a location",
    schema: z.object({
      location: z.string().describe("City or location name"),
    }),
  }
);

// ============================================================================
// Create Specialist Agents
// ============================================================================

const ragExpert = createReactAgent({
  llm: agentModel,
  tools: [searchDocuments, queryKnowledge],
  name: "rag_expert",
  prompt: `You are a RAG (Retrieval-Augmented Generation) expert.
Your job is to search documents and answer questions from the knowledge base.
Always cite your sources and be precise about what the documents say.
If you can't find relevant information, say so clearly.`,
});

const memoryExpert = createReactAgent({
  llm: agentModel,
  tools: [upsertMemory, recallMemory],
  name: "memory_expert",
  prompt: `You are a memory management expert.
Your job is to store and recall information about the user.
Save preferences, context, and important details the user shares.
When recalling, provide relevant memories that help personalize the experience.`,
});

const toolsExpert = createReactAgent({
  llm: agentModel,
  tools: [calculator, getCurrentTime, getWeather],
  name: "tools_expert",
  prompt: `You are a utility tools expert.
Your job is to perform calculations, get current time, check weather, and other utility tasks.
Be precise with calculations and format results clearly.`,
});

// ============================================================================
// Create Supervisor
// ============================================================================

const supervisorPrompt = `You are a supervisor managing a team of expert agents.
Your job is to route user requests to the most appropriate expert.

Available experts:
1. **rag_expert** - For knowledge/document queries, searching information, answering questions from docs
2. **memory_expert** - For storing/recalling user preferences, remembering context
3. **tools_expert** - For calculations, time, weather, and utility tasks

Guidelines:
- For document/knowledge questions → rag_expert
- For "remember this" or "what did I say about" → memory_expert
- For calculations, time, weather → tools_expert
- For complex requests, you may need to call multiple experts in sequence
- Simple greetings or clarifications can be handled directly without delegation

Always be helpful and ensure the user gets a complete answer.`;

const workflow = createSupervisor({
  agents: [ragExpert, memoryExpert, toolsExpert],
  llm: supervisorModel,
  prompt: supervisorPrompt,
  // "full_history" includes all agent messages, "last_message" only final response
  outputMode: "full_history",
});

// ============================================================================
// Compile with Memory Support
// ============================================================================

// Short-term memory (conversation persistence)
const checkpointer = new MemorySaver();

// Long-term memory (user preferences, facts)
const store = new InMemoryStore();

export const graph = workflow.compile({
  checkpointer,
  store,
});

graph.name = "SupervisorAgent";

// ============================================================================
// Usage Example
// ============================================================================

/*
const result = await graph.invoke(
  {
    messages: [{ role: "user", content: "Find docs about authentication and remember I'm working on the login feature" }],
  },
  {
    configurable: {
      thread_id: "user-session-123",
      userId: "user-456",
    },
  }
);
*/
```

### package.json

```json
{
  "name": "supervisor-agent",
  "version": "0.0.1",
  "description": "Supervisor agent orchestrating RAG, Memory, and Tools experts",
  "main": "src/index.ts",
  "type": "module",
  "scripts": {
    "build": "tsc",
    "dev": "tsx watch src/index.ts",
    "start": "tsx src/index.ts",
    "test": "vitest",
    "lint": "eslint src"
  },
  "dependencies": {
    "@langchain/anthropic": "^0.3.21",
    "@langchain/core": "^0.3.57",
    "@langchain/langgraph": "^0.3.0",
    "@langchain/langgraph-supervisor": "^0.0.1",
    "@langchain/openai": "^0.5.11",
    "zod": "^3.23.8"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.7.0",
    "vitest": "^2.0.0"
  }
}
```

### langgraph.json

```json
{
  "dependencies": ["."],
  "graphs": {
    "supervisor": "./src/graph.ts:graph"
  },
  "env": ".env"
}
```

---

## Approach 2: Manual StateGraph (Advanced)

For maximum control over routing logic, state management, and custom behavior.

<details>
<summary>Click to expand manual implementation</summary>

### State Definition

```typescript
// src/state.ts
import { Annotation, MessagesAnnotation } from "@langchain/langgraph";
import { BaseMessage } from "@langchain/core/messages";

export const RoutingDecision = Annotation.Root({
  primaryAgent: Annotation<"rag" | "memory" | "tools" | "direct">,
  secondaryAgents: Annotation<string[]>({
    default: () => [],
  }),
  reasoning: Annotation<string>,
});

export const SupervisorStateAnnotation = Annotation.Root({
  ...MessagesAnnotation.spec,
  currentAgent: Annotation<string | null>({ default: () => null }),
  routingDecision: Annotation<typeof RoutingDecision.State | null>({
    default: () => null,
  }),
  iterationCount: Annotation<number>({ default: () => 0 }),
});
```

### Graph Definition

```typescript
// src/graph.ts
import { StateGraph, START, END, Command } from "@langchain/langgraph";
import { ChatAnthropic } from "@langchain/anthropic";
import { SupervisorStateAnnotation } from "./state.js";
import { z } from "zod";

const model = new ChatAnthropic({ model: "claude-sonnet-4-20250514" });

// Routing schema for structured output
const RoutingSchema = z.object({
  next: z.enum(["rag_expert", "memory_expert", "tools_expert", "FINISH"]),
  reasoning: z.string(),
});

// Supervisor node - decides which agent to call
async function supervisor(state: typeof SupervisorStateAnnotation.State) {
  const systemPrompt = `You are a supervisor routing requests to experts.
Decide which expert should handle the request:
- rag_expert: document/knowledge queries
- memory_expert: store/recall user info
- tools_expert: calculations, time, weather
- FINISH: task complete, respond to user`;

  const response = await model
    .withStructuredOutput(RoutingSchema)
    .invoke([
      { role: "system", content: systemPrompt },
      ...state.messages,
    ]);

  if (response.next === "FINISH") {
    return new Command({ goto: END });
  }

  return new Command({
    goto: response.next,
    update: {
      currentAgent: response.next,
      routingDecision: {
        primaryAgent: response.next.replace("_expert", "") as any,
        secondaryAgents: [],
        reasoning: response.reasoning,
      },
    },
  });
}

// Import your existing agent graphs as subgraphs
import { graph as ragGraph } from "../../rag-agent/src/retrieval_graph/graph.js";
import { graph as memoryGraph } from "../../code-tester/src/memory_agent/graph.js";
// ... tools expert

const builder = new StateGraph(SupervisorStateAnnotation)
  .addNode("supervisor", supervisor)
  .addNode("rag_expert", ragGraph)      // Subgraph
  .addNode("memory_expert", memoryGraph) // Subgraph
  .addNode("tools_expert", toolsExpert)
  .addEdge(START, "supervisor")
  .addEdge("rag_expert", "supervisor")
  .addEdge("memory_expert", "supervisor")
  .addEdge("tools_expert", "supervisor");

export const graph = builder.compile();
```

</details>

---

## Data Flow Diagram

```
                              User Request
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                    createSupervisor() Internal Flow                          │
│                                                                               │
│  1. User message arrives                                                      │
│  2. Supervisor LLM receives message + available agent tools                  │
│  3. Supervisor decides: call agent tool OR respond directly                  │
│  4. If agent tool called:                                                    │
│     - Handoff message sent to agent                                          │
│     - Agent executes with its tools                                          │
│     - Agent returns result via transfer_back_to_supervisor                   │
│  5. Supervisor evaluates: need more agents? OR respond?                      │
│  6. Loop until supervisor responds directly                                  │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
                           Message Flow:
                                   │
    ┌──────────────────────────────┼──────────────────────────────┐
    │                              │                              │
    ▼                              ▼                              ▼
┌─────────────┐            ┌─────────────┐            ┌─────────────┐
│ RAG Expert  │            │Memory Expert│            │Tools Expert │
│             │            │             │            │             │
│ search_docs │            │upsert_memory│            │ calculator  │
│query_knowl. │            │recall_memory│            │ get_time    │
│             │            │             │            │ get_weather │
└─────────────┘            └─────────────┘            └─────────────┘
    │                              │                              │
    │              Vector DB       │    LangGraph      │  External │
    │              Retrieval       │    Store          │  APIs     │
    ▼                              ▼                              ▼
┌─────────────┐            ┌─────────────┐            ┌─────────────┐
│  Documents  │            │  Memories   │            │   Results   │
└─────────────┘            └─────────────┘            └─────────────┘
```

---

## Multi-Level Hierarchies

For complex systems, create supervisors that manage other supervisors:

```typescript
// Create team supervisors
const researchTeam = createSupervisor({
  agents: [ragExpert, webSearchExpert],
  llm: model,
  prompt: "You manage research tasks...",
}).compile({ name: "research_team" });

const productivityTeam = createSupervisor({
  agents: [memoryExpert, calendarExpert],
  llm: model,
  prompt: "You manage productivity tasks...",
}).compile({ name: "productivity_team" });

// Top-level supervisor manages teams
const topSupervisor = createSupervisor({
  agents: [researchTeam, productivityTeam, toolsExpert],
  llm: model,
  prompt: "You are the top-level coordinator...",
}).compile();
```

```
                         ┌─────────────────────┐
                         │   TOP SUPERVISOR    │
                         └──────────┬──────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
    ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
    │  RESEARCH TEAM  │   │PRODUCTIVITY TEAM│   │  TOOLS EXPERT   │
    │   (supervisor)  │   │   (supervisor)  │   │    (agent)      │
    └────────┬────────┘   └────────┬────────┘   └─────────────────┘
             │                     │
      ┌──────┴──────┐       ┌──────┴──────┐
      │             │       │             │
      ▼             ▼       ▼             ▼
  ┌───────┐   ┌─────────┐ ┌───────┐ ┌──────────┐
  │  RAG  │   │Web Search│ │Memory │ │ Calendar │
  └───────┘   └─────────┘ └───────┘ └──────────┘
```

---

## Comparison: Package vs Manual vs Google ADK

| Aspect | `createSupervisor()` | Manual StateGraph | Google ADK |
|--------|---------------------|-------------------|------------|
| **Setup Complexity** | Low (single function) | High (custom graph) | Low (Python class) |
| **Routing Control** | Automatic (LLM decides) | Full control | Automatic or manual |
| **State Management** | Built-in | Custom annotations | Session-based |
| **Memory** | checkpointer + store | Manual integration | Built-in session |
| **Parallelism** | Sequential by default | Custom parallel nodes | `ParallelAgent` |
| **Customization** | Prompt + outputMode | Everything | Extensive |
| **TypeScript Support** | Native | Native | Python only |
| **Best For** | Most use cases | Complex routing logic | GCP-native deployments |

---

## Deployment

### LangGraph Studio (Local Development)

```bash
# Start LangGraph Studio
npx @langchain/langgraph-cli dev
```

### GKE Deployment

```yaml
# k8s/supervisor-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: supervisor-agent
  namespace: agents
spec:
  replicas: 2
  selector:
    matchLabels:
      app: supervisor-agent
  template:
    metadata:
      labels:
        app: supervisor-agent
    spec:
      containers:
        - name: supervisor
          image: gcr.io/your-project/supervisor-agent:latest
          ports:
            - containerPort: 8123
          env:
            - name: ANTHROPIC_API_KEY
              valueFrom:
                secretKeyRef:
                  name: llm-secrets
                  key: anthropic-api-key
          resources:
            requests:
              memory: "512Mi"
              cpu: "250m"
            limits:
              memory: "1Gi"
              cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: supervisor-agent
  namespace: agents
spec:
  selector:
    app: supervisor-agent
  ports:
    - port: 80
      targetPort: 8123
```

### LangGraph Cloud

```bash
# Deploy to LangGraph Cloud
npx @langchain/langgraph-cli deploy
```

---

## Testing

```typescript
// tests/supervisor.test.ts
import { describe, it, expect } from "vitest";
import { graph } from "../src/graph.js";

describe("Supervisor Agent", () => {
  it("should route RAG queries to rag_expert", async () => {
    const result = await graph.invoke({
      messages: [{ role: "user", content: "What do the docs say about authentication?" }],
    });

    // Check that rag_expert was invoked
    const agentMessages = result.messages.filter(
      (m) => m.name === "rag_expert"
    );
    expect(agentMessages.length).toBeGreaterThan(0);
  });

  it("should route memory requests to memory_expert", async () => {
    const result = await graph.invoke({
      messages: [{ role: "user", content: "Remember that I prefer dark mode" }],
    });

    const agentMessages = result.messages.filter(
      (m) => m.name === "memory_expert"
    );
    expect(agentMessages.length).toBeGreaterThan(0);
  });

  it("should handle multi-step requests", async () => {
    const result = await graph.invoke({
      messages: [{
        role: "user",
        content: "Search for auth docs and remember I'm working on login"
      }],
    });

    // Both experts should be called
    const ragMessages = result.messages.filter((m) => m.name === "rag_expert");
    const memoryMessages = result.messages.filter((m) => m.name === "memory_expert");

    expect(ragMessages.length).toBeGreaterThan(0);
    expect(memoryMessages.length).toBeGreaterThan(0);
  });
});
```

---

## References

- [@langchain/langgraph-supervisor npm](https://www.npmjs.com/package/@langchain/langgraph-supervisor)
- [LangGraph Supervisor API Reference](https://langchain-ai.github.io/langgraphjs/reference/modules/langgraph-supervisor.html)
- [LangGraph Multi-Agent Tutorial](https://langchain-ai.github.io/langgraphjs/tutorials/multi_agent/agent_supervisor/)
- [createReactAgent Documentation](https://langchain-ai.github.io/langgraphjs/how-tos/create-react-agent/)
- [LangGraph Hierarchical Teams](https://langchain-ai.github.io/langgraph/tutorials/multi_agent/hierarchical_agent_teams/)
- [Google ADK Multi-Agent Systems](https://google.github.io/adk-docs/agents/multi-agents/)

# AI Agent Frameworks Comparison 2025

A comprehensive comparison of AI agent frameworks: Google ADK, AWS Bedrock Agents, Agno, LangGraph, CrewAI, Vercel AI SDK, TanStack AI, and AIPack.

## Executive Summary

| Framework | Primary Focus | Best For | Language | Maturity |
|-----------|---------------|----------|----------|----------|
| **Google ADK** | Enterprise multi-agent systems | Google Cloud deployments, enterprise | Python, TypeScript, Go | Early (2025) |
| **AWS Bedrock Agents** | Managed production infrastructure | AWS deployments, enterprise scaling | Python (+ any framework) | GA (2025) |
| **Agno** | Performance & speed | High-volume, real-time systems | Python | Mature |
| **LangGraph** | Stateful graph workflows | Complex workflows, production agents | Python, TypeScript | Most mature |
| **CrewAI** | Role-based collaboration | Rapid prototyping, team orchestration | Python | Mature |
| **Vercel AI SDK** | Frontend AI integration | Next.js/React apps, streaming | TypeScript | Mature (v5) |
| **TanStack AI** | Framework-agnostic AI | Multi-framework, isomorphic tools | TypeScript | Alpha (2025) |
| **AIPack** | Lightweight agentic runtime | Portable, shareable AI packs | Lua + Markdown | Early |

---

## 1. Google Agent Development Kit (ADK)

**Released:** Google Cloud NEXT 2025
**License:** Apache 2.0
**GitHub:** [google/adk-python](https://github.com/google/adk-python)

### Overview

Google ADK is an open-source, code-first framework for building, evaluating, and deploying sophisticated AI agents. It's the same framework powering Google products like Agentspace and Google Customer Engagement Suite.

### Key Features

- **Multi-Agent Orchestration**: Compose agents in parallel, sequential, or hierarchical workflows
- **Model Flexibility**: Works with Gemini, Vertex AI Model Garden, or any model via LiteLLM (Anthropic, Meta, Mistral, etc.)
- **Rich Tool Ecosystem**: Pre-built tools, MCP tools, third-party integrations (LangChain, LlamaIndex)
- **Bidirectional Streaming**: Audio and video streaming for multimodal dialogue
- **Agent Types**:
  - `LlmAgent` - Language model-powered reasoning
  - `SequentialAgent`, `ParallelAgent`, `LoopAgent` - Workflow controllers
- **Built-in Evaluation**: Create evaluation datasets and run locally
- **State Management**: Automatic short-term memory with integration points for long-term storage

### Deployment Options

- Local development with CLI and Developer UI
- Vertex AI Agent Engine (managed)
- Cloud Run / Docker (self-hosted)
- Any containerized environment

### Strengths

- Deep Google Cloud integration
- Enterprise-grade security and scalability
- Model-agnostic despite Google optimization
- Comprehensive developer tooling (CLI, Dev UI, evaluation)

### Weaknesses

- Early-stage developer experience
- Optimized primarily for GCP ecosystem
- Smaller community compared to LangChain ecosystem

### Best For

- Organizations already invested in Google Cloud
- Enterprise deployments requiring compliance features
- Teams needing bidirectional streaming (audio/video)

---

## 2. AWS Bedrock Agents & AgentCore

**Released:** Multi-Agent GA March 2025, AgentCore GA 2025
**Service:** Fully managed AWS service
**Docs:** [AWS Bedrock Agents](https://docs.aws.amazon.com/bedrock/latest/userguide/agents.html)

### Overview

Amazon Bedrock Agents is a fully managed service for building, deploying, and scaling AI agents on AWS. With the addition of AgentCore (GA 2025), AWS provides production infrastructure that works with any agent framework—CrewAI, LangGraph, Google ADK, and more.

### Key Features

- **Multi-Agent Collaboration** (GA March 2025):
  - Supervisor agent coordinates multiple collaborator agents
  - Two modes: Supervisor Mode and Supervisor with Routing Mode
  - Parallel communication for efficient task completion
- **AgentCore**:
  - Framework-agnostic runtime (supports CrewAI, LangGraph, LlamaIndex, Strands, etc.)
  - Handles scaling, security, and infrastructure
  - Quality evaluations and policy controls
- **Inline Agents**: Dynamically adjust agent roles at runtime
- **Payload Referencing**: Reduce data transfer by referencing linked data
- **Infrastructure as Code**: CloudFormation and CDK support
- **Enhanced Traceability**: CloudWatch integration, sub-step tracking
- **Knowledge Bases**: Native integration with Amazon Bedrock Knowledge Bases
- **Guardrails**: Built-in safety controls and content filtering
- **Action Groups**: Connect agents to Lambda functions and APIs

### Deployment Options

- Amazon Bedrock (fully managed)
- AgentCore for framework-agnostic deployment
- Private VPC networking
- Cross-account deployment via CloudFormation

### Strengths

- Best-in-class AWS integration (Lambda, S3, DynamoDB, etc.)
- Enterprise-grade security, compliance, and governance
- Framework-agnostic with AgentCore (use any framework)
- Automatic scaling to thousands of users
- Native multi-agent orchestration
- Pay-per-use pricing model

### Weaknesses

- AWS vendor lock-in
- Less control compared to self-hosted frameworks
- Requires AWS ecosystem knowledge
- Some features still in preview

### Best For

- Organizations committed to AWS
- Enterprise production workloads needing managed scaling
- Teams wanting to use open-source frameworks with managed infrastructure
- Regulated industries requiring compliance (HIPAA, SOC2, etc.)

---

## 3. Agno (formerly Phidata)

**License:** Open Source
**GitHub:** [agno-agi/agno](https://github.com/agno-agi/agno)
**Website:** [agno.com](https://www.agno.com)

### Overview

Agno is the unified stack for building, running, and managing multi-agent systems. It emphasizes performance, composability, and clean integration without boilerplate orchestration layers.

### Key Features

- **Extreme Performance**:
  - 529× faster instantiation than LangGraph
  - 70× faster than CrewAI
  - 24× lower memory than LangGraph
  - 10× lower memory than CrewAI
- **Multi-Modal Support**: Text, image, audio, video inputs/outputs
- **Structured I/O**: `input_schema` and `output_schema` for predictable behavior
- **23+ Model Providers**: Direct unified interfaces
- **Team Collaboration Modes**:
  - Coordinator Mode (central orchestrator)
  - Direct delegation
- **AgentOS Platform**: Enterprise control plane that runs in your cloud

### Deployment Options

- Local development
- AgentOS in your private cloud
- Self-hosted infrastructure
- No data sent to external services (complete privacy)

### Strengths

- Best-in-class performance and memory efficiency
- Strong type validation
- Clean, composable architecture
- Enterprise AgentOS with private cloud deployment
- Extensive out-of-the-box toolkits (100+)

### Weaknesses

- Smaller community than LangChain/LangGraph
- Less documentation compared to established frameworks
- Relatively newer rebrand from Phidata

### Best For

- High-volume, real-time systems
- Performance-critical applications
- Teams prioritizing memory management
- Multimodal agent applications

---

## 4. LangGraph

**Released:** v1.0 November 2025
**License:** MIT
**GitHub:** [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**Website:** [langchain.com/langgraph](https://www.langchain.com/langgraph)

### Overview

LangGraph is a low-level framework for building stateful agents as graph-based programs. It models agents as finite state machines where nodes represent reasoning/tool-use steps and edges define transitions.

### Key Features

- **Graph-Based Architecture**: Supports loops, cycles, and revisiting states (unlike DAG-only chains)
- **Robust State Management**: Persistent state across sessions
- **Time-Travel Debugging**: Replay and inspect agent execution
- **Human-in-the-Loop**: Built-in interrupt and approval workflows
- **Fault Tolerance**: Automatic retry and error recovery
- **LangSmith Integration**: Observability, tracing, performance monitoring
- **Control Flows**: Single agent, multi-agent, hierarchical, sequential

### Deployment Options

- Local development
- LangGraph Platform (1-click SaaS deployment)
- LangGraph Cloud
- Self-hosted with Docker/Kubernetes
- LangGraph Studio v2 for debugging

### Strengths

- Most mature and battle-tested framework
- Extensive community and ecosystem
- Excellent observability with LangSmith
- Lowest latency in benchmarks (reduced context passing)
- Best for complex, conditional workflows

### Weaknesses

- Steeper learning curve than CrewAI
- Heavier memory footprint than Agno
- Can be overkill for simple use cases

### Best For

- Complex, stateful production agents
- Workflows with loops and conditional logic
- Teams needing strong observability
- Enterprise deployments requiring fault tolerance

---

## 5. CrewAI

**License:** MIT
**GitHub:** [crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**Stars:** 30,500+ | **Downloads:** 1M+ monthly
**Website:** [crewai.com](https://www.crewai.com)

### Overview

CrewAI is a framework for orchestrating role-playing, autonomous AI agents. Using a "crew" metaphor, it makes multi-agent interactions approachable by defining specialized agent roles that collaborate on tasks.

### Key Features

- **Role-Based Architecture**:
  - Define agents with roles, goals, and capabilities
  - Agents collaborate as a "crew"
- **Dual Workflow Management**:
  - Crews (autonomous collaboration)
  - Flows (precise control)
- **Core Components**:
  - Agents → Tasks → Crews → Tools → Processes
- **Planning Feature**: Auto-generates step-by-step workflow before execution
- **100,000+ Certified Developers**: Strong community support
- **Framework Independence**: Not dependent on LangChain

### Deployment Options

- Local development
- CrewAI Enterprise (managed platform)
- Self-hosted with standard Python deployment
- Docker containerization

### Strengths

- Most intuitive API for multi-agent systems
- Excellent for rapid prototyping
- Low code, instant feedback
- Great for creative, multi-perspective tasks
- Strong documentation

### Weaknesses

- Less granular control than LangGraph
- Challenges with smaller (7B) open-source models
- Lacks built-in monitoring for production
- Manual implementation needed for error recovery at scale

### Best For

- Rapid prototyping of multi-agent systems
- Content creation and research workflows
- Teams wanting intuitive role-based design
- Quick proof-of-concepts

---

## 6. Vercel AI SDK

**Released:** AI SDK 5 - July 2025
**License:** Apache 2.0
**GitHub:** [vercel/ai](https://github.com/vercel/ai)
**Docs:** [ai-sdk.dev](https://ai-sdk.dev/docs/introduction)

### Overview

The AI SDK is Vercel's TypeScript toolkit for building AI-powered applications and agents. Designed for React, Next.js, Vue, Svelte, and Node.js, it provides a unified API across all major model providers with first-class streaming and edge runtime support.

### Key Features

- **Multi-Step Agent Loops**:
  - `stopWhen` - Define when tool-calling loop stops
  - `prepareStep` - Control settings for each step (model, messages, system prompt)
  - `maxSteps` - Limit maximum iterations
- **Tool System**:
  - Type-safe tool definitions with Zod schemas
  - Automatic tool result handling
  - Tool-level provider options (e.g., Anthropic caching)
- **Streaming First**: Built-in streaming with `streamText` and `streamObject`
- **Provider Agnostic**: OpenAI, Anthropic, Google, Mistral, Cohere, and more
- **React Hooks**: `useChat`, `useCompletion`, `useAssistant`
- **Structured Output**: `generateObject` with schema validation

### Agent Example

```typescript
import { generateText, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateText({
  model: openai('gpt-4o'),
  system: 'You are a helpful customer support agent.',
  prompt: userQuery,

  tools: {
    lookupOrder: tool({
      description: 'Look up order status by order ID',
      parameters: z.object({
        orderId: z.string().describe('The order ID to look up'),
      }),
      execute: async ({ orderId }) => {
        return await db.orders.findById(orderId);
      },
    }),
    createTicket: tool({
      description: 'Create a support ticket',
      parameters: z.object({
        subject: z.string(),
        priority: z.enum(['low', 'medium', 'high']),
      }),
      execute: async ({ subject, priority }) => {
        return await ticketService.create({ subject, priority });
      },
    }),
  },

  // Agent loop configuration
  maxSteps: 10,
  stopWhen: stepCountIs(10),

  // Dynamic step preparation
  prepareStep: async ({ previousSteps, stepCount }) => {
    // Switch to cheaper model after initial reasoning
    if (stepCount > 3) {
      return { model: openai('gpt-4o-mini') };
    }
    return {};
  },
});
```

### Deployment Options

- Vercel (optimized with Fluid Compute)
- Any Node.js server
- Edge runtimes (Cloudflare Workers, Deno)
- Serverless (AWS Lambda, etc.)

### Strengths

- Best TypeScript/React integration
- Excellent streaming performance
- Simple API, easy to learn
- First-class Next.js support
- Edge runtime compatible
- Great for frontend-focused teams

### Weaknesses

- Less suited for complex multi-agent orchestration
- No built-in state persistence (like LangGraph checkpoints)
- Limited to TypeScript/JavaScript ecosystem
- No native multi-agent coordination

### Best For

- Next.js / React applications
- TypeScript-first teams
- Simple to medium complexity agents
- Streaming chat interfaces
- Edge deployments

---

## 7. TanStack AI

**Released:** Alpha - December 2025
**License:** MIT
**GitHub:** [TanStack/ai](https://github.com/TanStack/ai)
**Docs:** [tanstack.com/ai](https://tanstack.com/ai/latest/docs)

### Overview

TanStack AI is a framework-agnostic, open-source AI SDK with a unified interface across multiple providers. Built by the creators of TanStack Query/Router, it emphasizes type safety, no vendor lock-in, and isomorphic tool execution.

### Key Features

- **Isomorphic Tools**: Define once, run on server or client
  ```typescript
  const weatherTool = toolDefinition({
    name: 'getWeather',
    input: z.object({ city: z.string() }),
    output: z.object({ temp: z.number(), conditions: z.string() }),
  })
    .server(async ({ city }) => fetchWeatherAPI(city))
    .client(async ({ city }) => getCachedWeather(city));
  ```
- **Agent Loop Strategies**: Built-in chat completion and agent loops
- **Tool Approval Workflows**: Human-in-the-loop for tool execution
- **Headless Chat State**: Framework-agnostic state management
- **Stream Adapters**: SSE, HTTP streams, custom adapters
- **Provider Agnostic**: OpenAI, Anthropic, Gemini, Ollama + custom

### Agent Example

```typescript
import { createChat, toolDefinition } from '@tanstack/ai';
import { z } from 'zod';

// Define isomorphic tools
const searchKnowledgeBase = toolDefinition({
  name: 'searchKB',
  description: 'Search the knowledge base for answers',
  input: z.object({ query: z.string() }),
  output: z.object({ results: z.array(z.string()) }),
}).server(async ({ query }) => {
  return await vectorDB.search(query);
});

const escalateToHuman = toolDefinition({
  name: 'escalate',
  description: 'Escalate to human agent',
  input: z.object({ reason: z.string() }),
  output: z.object({ ticketId: z.string() }),
  requiresApproval: true,  // Human-in-the-loop
}).server(async ({ reason }) => {
  return await ticketSystem.escalate(reason);
});

// Create chat with agent capabilities
const chat = createChat({
  provider: 'openai',
  model: 'gpt-4o',
  tools: [searchKnowledgeBase, escalateToHuman],
  agentLoop: true,  // Enable multi-step agent
  maxIterations: 5,
});

// Use with any framework
const response = await chat.send('How do I reset my password?');
```

### Framework Integrations

- React (`@tanstack/ai-react`)
- SolidJS (`@tanstack/ai-solid`)
- Vanilla JS
- Svelte (planned)
- Vue (planned)

### Strengths

- True framework agnostic (unlike Vercel AI SDK's React focus)
- Isomorphic tools (same definition for client/server)
- Built-in tool approval workflows
- No vendor lock-in philosophy
- Type-safe throughout
- From trusted TanStack ecosystem

### Weaknesses

- Still in alpha (December 2025)
- Smaller community than Vercel AI SDK
- Less production-tested
- Documentation still evolving
- No managed deployment platform

### Best For

- Multi-framework projects (React + SolidJS + vanilla)
- Teams avoiding vendor lock-in
- Projects needing isomorphic tool execution
- Those already using TanStack ecosystem

---

## 8. AIPack

**License:** Open Source
**Website:** [aipack.ai](https://aipack.ai)

### Overview

AIPack is a lightweight, open-source agentic runtime for running, building, and sharing AI Packs. It's designed to be minimal (<20MB), dependency-free, and portable.

### Key Features

- **AI Packs**: `.aipack` cross-platform files containing:
  - Agent Files (`.aip` multi-stage markdown)
  - Logic (Lua scripts)
  - Data (markdown, JSON, CSV, SQLite)
- **Parallel Execution**: Built-in parallelism for multiple inputs
- **Map-Reduce Stages**: Before All / After All processing patterns
- **Model Agnostic**: Supports all major AI providers
- **IDE Agnostic**: Works anywhere

### Deployment Options

- Local execution
- Cloud/server deployment
- Serverless environments
- Completely portable `.aipack` files

### Strengths

- Extremely lightweight (<20MB, zero dependencies)
- Highly portable and shareable
- Simple file-based agent definition
- Built-in parallelism
- Works offline

### Weaknesses

- Less feature-rich than other frameworks
- Smaller community and ecosystem
- Lua-based logic (different from Python ecosystem)
- Early-stage development

### Best For

- Lightweight, portable agent deployment
- Sharing agents as distributable packages
- Resource-constrained environments
- Simple agent workflows

---

## Feature Comparison Matrix

| Feature | Google ADK | AWS Bedrock | Agno | LangGraph | CrewAI | Vercel AI SDK | TanStack AI | AIPack |
|---------|------------|-------------|------|-----------|--------|---------------|-------------|--------|
| **Multi-Agent Support** | ✅ Hierarchical | ✅ Supervisor/Collab | ✅ Teams | ✅ Graph-based | ✅ Crews | ❌ Single | ❌ Single | ✅ Basic |
| **State Management** | ✅ Built-in | ✅ Managed | ✅ Multiple | ✅ Persistent | ✅ Structured | ⚠️ Manual | ✅ Headless | ❌ Limited |
| **Human-in-the-Loop** | ✅ | ✅ | ✅ | ✅ Native | ✅ | ⚠️ Manual | ✅ Built-in | ❌ |
| **Streaming** | ✅ Bidirectional | ✅ | ✅ | ✅ | ✅ | ✅ Excellent | ✅ SSE/HTTP | ❌ |
| **Evaluation Tools** | ✅ Built-in | ✅ AgentCore | ⚠️ External | ✅ LangSmith | ⚠️ External | ⚠️ External | ⚠️ External | ❌ |
| **Multi-Modal** | ✅ | ✅ Via Bedrock | ✅ Native | ⚠️ Via tools | ⚠️ Via tools | ✅ Images | ⚠️ Limited | ⚠️ Limited |
| **MCP Support** | ✅ | ⚠️ Limited | ✅ | ✅ | ⚠️ Limited | ❌ | ❌ | ❌ |
| **Enterprise Features** | ✅ Strong | ✅ Best | ✅ AgentOS | ✅ LangSmith | ⚠️ Basic | ⚠️ Vercel | ❌ | ❌ |
| **Knowledge Bases** | ⚠️ External | ✅ Native | ⚠️ External | ⚠️ External | ⚠️ External | ❌ | ❌ | ❌ |
| **Guardrails** | ✅ Callbacks | ✅ Native | ✅ Built-in | ⚠️ External | ✅ Task-level | ⚠️ Manual | ⚠️ Manual | ❌ |
| **Edge Runtime** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Native | ✅ Native | ❌ |
| **React Hooks** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Native | ✅ Native | ❌ |
| **Learning Curve** | Medium | Medium | Low-Medium | High | Low | Low | Low | Low |

---

## LangGraph: Complex Workflow Examples

These patterns are either **unique to LangGraph** or significantly harder to implement in other frameworks.

### 1. Cyclic Refinement Loop (Self-Correction)

**Why LangGraph**: Other frameworks use DAG (Directed Acyclic Graph) which cannot loop back. LangGraph supports true cycles.

```python
from langgraph.graph import StateGraph, END
from typing import TypedDict, Literal

class State(TypedDict):
    draft: str
    feedback: str
    revision_count: int
    approved: bool

def writer(state: State) -> State:
    # LLM generates/revises content based on feedback
    return {"draft": generate_draft(state["draft"], state["feedback"])}

def critic(state: State) -> State:
    # LLM reviews and provides feedback
    feedback, approved = review_draft(state["draft"])
    return {
        "feedback": feedback,
        "approved": approved,
        "revision_count": state["revision_count"] + 1
    }

def should_continue(state: State) -> Literal["writer", "end"]:
    # Loop until approved OR max 5 revisions
    if state["approved"] or state["revision_count"] >= 5:
        return "end"
    return "writer"  # CYCLE BACK - impossible in DAG frameworks

graph = StateGraph(State)
graph.add_node("writer", writer)
graph.add_node("critic", critic)
graph.add_edge("writer", "critic")
graph.add_conditional_edges("critic", should_continue, {"writer": "writer", "end": END})
```

**Cannot achieve in CrewAI/Bedrock**: They don't support true cycles—you'd need hacky workarounds with recursive task spawning.

---

### 2. Time-Travel Debugging & Branching

**Why LangGraph**: Built-in checkpointing allows rewinding to any state and creating alternative execution branches.

```python
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph

# Compile with checkpointer
checkpointer = MemorySaver()
app = graph.compile(checkpointer=checkpointer)

# Run workflow
config = {"configurable": {"thread_id": "user-123"}}
result = app.invoke({"query": "Analyze sales data"}, config)

# TIME TRAVEL: Get all checkpoints
history = list(app.get_state_history(config))

# Rewind to step 3 and try different path
past_state = history[3]
app.update_state(config, {"strategy": "alternative_approach"}, as_node="planner")

# Fork execution from that point
new_result = app.invoke(None, config)  # Continues from modified state
```

**Use case**: User says "go back to before you called the API and try a different approach"

**Cannot achieve in CrewAI/Agno**: No built-in state history or replay capability.

---

### 3. Human-in-the-Loop with State Inspection

**Why LangGraph**: Native interrupt points with full state visibility and modification.

```python
from langgraph.graph import StateGraph, END

class State(TypedDict):
    messages: list
    pending_action: dict
    human_approved: bool

def plan_action(state: State) -> State:
    action = llm_plan_action(state["messages"])
    return {"pending_action": action}

def execute_action(state: State) -> State:
    result = execute(state["pending_action"])
    return {"messages": state["messages"] + [result]}

def check_approval(state: State) -> Literal["execute", "replan"]:
    if state["human_approved"]:
        return "execute"
    return "replan"

graph = StateGraph(State)
graph.add_node("plan", plan_action)
graph.add_node("execute", execute_action)
graph.add_conditional_edges("plan", check_approval)

# INTERRUPT before dangerous actions
app = graph.compile(
    checkpointer=MemorySaver(),
    interrupt_before=["execute"]  # Pause here for human review
)

# Execution pauses at "execute" node
result = app.invoke({"messages": ["Delete all user data"]}, config)
# result.status == "interrupted"

# Human reviews state["pending_action"], then:
app.update_state(config, {"human_approved": True})
final = app.invoke(None, config)  # Continues execution
```

**Use case**: AI agent must get approval before making API calls, database changes, or sending emails.

**Harder in other frameworks**: CrewAI has HITL but no state inspection. Bedrock requires Lambda/Step Functions integration.

---

### 4. Parallel Fan-Out with Convergence

**Why LangGraph**: Native support for parallel execution with typed state merging.

```python
from langgraph.graph import StateGraph, END
from operator import add
from typing import Annotated

class ResearchState(TypedDict):
    query: str
    # Results from parallel researchers are MERGED via `add`
    findings: Annotated[list[str], add]
    final_report: str

def researcher_web(state: ResearchState) -> ResearchState:
    return {"findings": [search_web(state["query"])]}

def researcher_academic(state: ResearchState) -> ResearchState:
    return {"findings": [search_arxiv(state["query"])]}

def researcher_internal(state: ResearchState) -> ResearchState:
    return {"findings": [search_company_docs(state["query"])]}

def synthesizer(state: ResearchState) -> ResearchState:
    # All findings merged automatically
    report = synthesize(state["findings"])  # Has all 3 sources
    return {"final_report": report}

graph = StateGraph(ResearchState)
graph.add_node("web", researcher_web)
graph.add_node("academic", researcher_academic)
graph.add_node("internal", researcher_internal)
graph.add_node("synthesize", synthesizer)

# Fan-out: run 3 researchers in PARALLEL
graph.add_edge("__start__", "web")
graph.add_edge("__start__", "academic")
graph.add_edge("__start__", "internal")

# Fan-in: all converge to synthesizer
graph.add_edge("web", "synthesize")
graph.add_edge("academic", "synthesize")
graph.add_edge("internal", "synthesize")
graph.add_edge("synthesize", END)
```

**Cannot achieve cleanly in CrewAI**: CrewAI processes tasks sequentially by default. Parallel requires workarounds.

---

### 5. Subgraph Composition (Nested Workflows)

**Why LangGraph**: Graphs can be nested as nodes in parent graphs with isolated state.

```python
# Define reusable subgraph for code review
code_review_graph = StateGraph(CodeReviewState)
code_review_graph.add_node("lint", run_linter)
code_review_graph.add_node("test", run_tests)
code_review_graph.add_node("security", security_scan)
code_review_graph.add_edge("lint", "test")
code_review_graph.add_edge("test", "security")
code_review_subgraph = code_review_graph.compile()

# Parent graph uses subgraph as a node
main_graph = StateGraph(MainState)
main_graph.add_node("plan", planning_agent)
main_graph.add_node("code", coding_agent)
main_graph.add_node("review", code_review_subgraph)  # Nested graph!
main_graph.add_node("deploy", deployment_agent)

main_graph.add_edge("plan", "code")
main_graph.add_edge("code", "review")
main_graph.add_conditional_edges("review", check_review_passed)
```

**Use case**: Build complex multi-stage pipelines from reusable components.

---

### 6. Fault-Tolerant Execution with Automatic Recovery

**Why LangGraph**: Checkpointing enables automatic resume from last successful state.

```python
from langgraph.checkpoint.postgres import PostgresSaver

# Production-grade persistence
checkpointer = PostgresSaver.from_conn_string(DATABASE_URL)
app = graph.compile(checkpointer=checkpointer)

try:
    result = app.invoke(input_data, config)
except Exception as e:
    # Get last successful checkpoint
    last_state = app.get_state(config)
    print(f"Failed at node: {last_state.next}")
    print(f"Completed nodes: {last_state.values}")

    # Fix the issue, then RESUME (not restart)
    result = app.invoke(None, config)  # Continues from checkpoint
```

**Use case**: Long-running workflows (data pipelines, multi-step analysis) that shouldn't restart from scratch on failure.

---

### 7. Dynamic Tool Selection with Retry Logic

**Why LangGraph**: Conditional edges + cycles enable sophisticated error handling.

```python
class State(TypedDict):
    query: str
    tool_attempts: dict[str, int]
    result: str | None
    error: str | None

def select_tool(state: State) -> Literal["api_a", "api_b", "api_c", "fail"]:
    # Try tools in order, skip if max retries exceeded
    for tool in ["api_a", "api_b", "api_c"]:
        if state["tool_attempts"].get(tool, 0) < 3:
            return tool
    return "fail"

def handle_result(state: State) -> Literal["success", "retry"]:
    if state["error"]:
        return "retry"  # Go back to tool selection
    return "success"

graph = StateGraph(State)
graph.add_node("api_a", call_api_a)
graph.add_node("api_b", call_api_b)
graph.add_node("api_c", call_api_c)
graph.add_node("process", process_result)

graph.add_conditional_edges("__start__", select_tool)
for api in ["api_a", "api_b", "api_c"]:
    graph.add_conditional_edges(api, handle_result, {
        "success": "process",
        "retry": "__start__"  # CYCLE back to try next tool
    })
```

---

## Comparison: What Each Framework Cannot Do

| Workflow Pattern | LangGraph | CrewAI | AWS Bedrock | Google ADK | Agno |
|------------------|-----------|--------|-------------|------------|------|
| **True Cycles** | ✅ Native | ❌ No | ⚠️ Step Functions | ⚠️ LoopAgent | ⚠️ Limited |
| **Time Travel** | ✅ Native | ❌ No | ❌ No | ❌ No | ❌ No |
| **State Branching** | ✅ Fork from any point | ❌ No | ❌ No | ❌ No | ❌ No |
| **Interrupt & Resume** | ✅ Any node | ⚠️ Basic | ⚠️ Lambda waits | ⚠️ Basic | ⚠️ Basic |
| **State Inspection** | ✅ Full history | ❌ No | ⚠️ CloudWatch | ⚠️ Limited | ⚠️ Limited |
| **Nested Subgraphs** | ✅ Native | ❌ No | ⚠️ Nested workflows | ⚠️ Limited | ❌ No |
| **Typed State Merging** | ✅ Annotated reducers | ❌ No | ❌ No | ❌ No | ❌ No |

---

---

## Guardrails Comparison

Yes, **all frameworks support guardrails**, but implementation varies significantly.

### Guardrails Support Matrix

| Framework | Built-in Guardrails | External Integration | Custom Guardrails | Enterprise |
|-----------|---------------------|---------------------|-------------------|------------|
| **AWS Bedrock** | ✅ Native (best) | N/A | ✅ | ✅ Full suite |
| **Google ADK** | ✅ Gemini safety | ✅ Plugins | ✅ Callbacks | ✅ |
| **Agno** | ✅ PII, Injection, Moderation | ✅ Guardrails AI | ✅ BaseGuardrail | ✅ |
| **LangGraph** | ⚠️ Via middleware | ✅ NeMo, Guardrails AI | ✅ Custom nodes | ✅ |
| **CrewAI** | ✅ Task guardrails | ✅ Bedrock Guardrails | ✅ Functions | ⚠️ Enterprise only |
| **AIPack** | ❌ None | ❌ | ⚠️ Lua scripts | ❌ |

---

### 1. AWS Bedrock Guardrails (Best Native Support)

**Native, fully managed guardrails** - no code required.

```python
import boto3

# Create guardrail via AWS Console or API
bedrock = boto3.client('bedrock')

guardrail = bedrock.create_guardrail(
    name='customer-support-guardrail',
    description='Guardrails for customer support agent',

    # Content filters
    contentPolicyConfig={
        'filtersConfig': [
            {'type': 'HATE', 'inputStrength': 'HIGH', 'outputStrength': 'HIGH'},
            {'type': 'VIOLENCE', 'inputStrength': 'HIGH', 'outputStrength': 'HIGH'},
            {'type': 'SEXUAL', 'inputStrength': 'HIGH', 'outputStrength': 'HIGH'},
            {'type': 'MISCONDUCT', 'inputStrength': 'MEDIUM', 'outputStrength': 'HIGH'},
        ]
    },

    # PII detection and masking
    sensitiveInformationPolicyConfig={
        'piiEntitiesConfig': [
            {'type': 'EMAIL', 'action': 'ANONYMIZE'},
            {'type': 'PHONE', 'action': 'ANONYMIZE'},
            {'type': 'SSN', 'action': 'BLOCK'},
            {'type': 'CREDIT_DEBIT_CARD_NUMBER', 'action': 'BLOCK'},
        ]
    },

    # Topic blocking
    topicPolicyConfig={
        'topicsConfig': [
            {
                'name': 'competitor-discussion',
                'definition': 'Discussions comparing our product to competitors',
                'action': 'BLOCK'
            },
            {
                'name': 'legal-advice',
                'definition': 'Requests for legal or medical advice',
                'action': 'BLOCK'
            }
        ]
    },

    # Word filters
    wordPolicyConfig={
        'wordsConfig': [{'text': 'lawsuit'}],
        'managedWordListsConfig': [{'type': 'PROFANITY'}]
    }
)

# Apply to agent
agent = bedrock.create_agent(
    agentName='support-agent',
    guardrailConfiguration={
        'guardrailIdentifier': guardrail['guardrailId'],
        'guardrailVersion': 'DRAFT'
    }
)
```

**Bedrock Guardrails Features:**
- ✅ Content filtering (hate, violence, sexual, misconduct)
- ✅ PII detection & anonymization
- ✅ Topic blocking (custom denied topics)
- ✅ Word/phrase filtering
- ✅ Contextual grounding (hallucination prevention)
- ✅ No code required - configure via Console

---

### 2. Google ADK Guardrails

**Multi-layered approach** with callbacks and plugins.

```python
from google.adk.agents import Agent
from google.adk.runners import Runner
from google.genai.types import Part

# Method 1: Gemini built-in safety settings
agent = Agent(
    name="support_agent",
    model="gemini-2.0-flash",
    generate_content_config={
        "safety_settings": [
            {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_LOW_AND_ABOVE"},
            {"category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_MEDIUM_AND_ABOVE"},
            {"category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_LOW_AND_ABOVE"},
            {"category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_MEDIUM_AND_ABOVE"},
        ]
    }
)

# Method 2: Custom callback guardrails
def input_guardrail(callback_context, llm_request):
    """Block requests containing sensitive patterns."""
    user_input = str(llm_request.contents[-1])

    # Check for prompt injection
    injection_patterns = ["ignore previous", "disregard instructions", "you are now"]
    for pattern in injection_patterns:
        if pattern.lower() in user_input.lower():
            # Return blocked response instead of calling LLM
            return Part.from_text("I cannot process that request.")

    # Check for PII
    if contains_ssn(user_input) or contains_credit_card(user_input):
        return Part.from_text("Please don't share sensitive personal information.")

    return None  # Continue to LLM

def output_guardrail(callback_context, llm_response):
    """Validate and filter LLM outputs."""
    response_text = str(llm_response.content)

    # Block competitor mentions
    competitors = ["competitor_a", "competitor_b"]
    for comp in competitors:
        if comp.lower() in response_text.lower():
            return Part.from_text("I can only discuss our products and services.")

    return None  # Allow response

# Apply guardrails via callbacks
agent = Agent(
    name="safe_agent",
    model="gemini-2.0-flash",
    before_model_callback=input_guardrail,
    after_model_callback=output_guardrail,
)

# Method 3: Use Gemini as a safety judge (LLM-as-guardrail)
safety_judge = Agent(
    name="safety_judge",
    model="gemini-2.0-flash-lite",  # Fast, cheap model
    instruction="""
    Evaluate if the following content is safe. Check for:
    - Harmful content (violence, hate speech)
    - PII exposure
    - Off-topic responses
    - Hallucinations

    Respond with JSON: {"safe": true/false, "reason": "..."}
    """
)
```

---

### 3. Agno Guardrails

**Built-in guardrails** + custom extensibility.

```python
from agno import Agent, Team
from agno.guardrails import (
    PIIDetectionGuardrail,
    PromptInjectionGuardrail,
    OpenAIModerationGuardrail,
    BaseGuardrail,
    GuardrailResult
)

# Built-in guardrails
pii_guardrail = PIIDetectionGuardrail(
    detect_emails=True,
    detect_phone_numbers=True,
    detect_ssn=True,
    detect_credit_cards=True,
    action="block"  # or "mask"
)

injection_guardrail = PromptInjectionGuardrail(
    sensitivity="high"
)

moderation_guardrail = OpenAIModerationGuardrail(
    categories=["hate", "violence", "sexual", "self-harm"]
)

# Custom guardrail
class CompetitorGuardrail(BaseGuardrail):
    def __init__(self, competitors: list[str]):
        self.competitors = [c.lower() for c in competitors]

    def check(self, content: str) -> GuardrailResult:
        content_lower = content.lower()
        for competitor in self.competitors:
            if competitor in content_lower:
                return GuardrailResult(
                    passed=False,
                    message=f"Content mentions competitor: {competitor}"
                )
        return GuardrailResult(passed=True)

# Apply to agent
agent = Agent(
    name="SupportAgent",
    model="gpt-4o",
    guardrails=[
        pii_guardrail,
        injection_guardrail,
        moderation_guardrail,
        CompetitorGuardrail(["acme", "globex", "initech"])
    ],
    input_guardrails=[pii_guardrail, injection_guardrail],  # Input only
    output_guardrails=[moderation_guardrail],  # Output only
)

# Or apply to entire team
team = Team(
    agents=[agent1, agent2, agent3],
    guardrails=[pii_guardrail, injection_guardrail]  # Applied to all
)
```

---

### 4. LangGraph Guardrails

**Implemented via middleware, custom nodes, or external libraries.**

```python
from langgraph.graph import StateGraph, END
from guardrails import Guard
from guardrails.hub import ToxicLanguage, DetectPII, CompetitorCheck
from nemo_guardrails import RailsConfig, LLMRails

# Method 1: Guardrails AI integration
guard = Guard().use_many(
    ToxicLanguage(on_fail="exception"),
    DetectPII(on_fail="fix"),  # Masks PII automatically
    CompetitorCheck(competitors=["acme", "globex"], on_fail="exception"),
)

class State(TypedDict):
    messages: list
    guardrail_passed: bool
    blocked_reason: str | None

def input_guardrail_node(state: State) -> State:
    """Validate input before processing."""
    user_message = state["messages"][-1]

    try:
        guard.validate(user_message)
        return {"guardrail_passed": True, "blocked_reason": None}
    except Exception as e:
        return {"guardrail_passed": False, "blocked_reason": str(e)}

def should_continue(state: State) -> str:
    if state["guardrail_passed"]:
        return "process"
    return "blocked"

def process_node(state: State) -> State:
    # Your agent logic here
    response = llm.invoke(state["messages"])

    # Validate output
    try:
        guard.validate(response)
        return {"messages": state["messages"] + [response]}
    except Exception:
        return {"messages": state["messages"] + ["I cannot respond to that."]}

def blocked_node(state: State) -> State:
    return {"messages": state["messages"] + [f"Blocked: {state['blocked_reason']}"]}

graph = StateGraph(State)
graph.add_node("input_guard", input_guardrail_node)
graph.add_node("process", process_node)
graph.add_node("blocked", blocked_node)

graph.add_edge("__start__", "input_guard")
graph.add_conditional_edges("input_guard", should_continue)
graph.add_edge("process", END)
graph.add_edge("blocked", END)

# Method 2: NVIDIA NeMo Guardrails
config = RailsConfig.from_path("./guardrails_config")
rails = LLMRails(config)

# Wrap your LangGraph with NeMo
from nemo_guardrails.integrations.langchain.runnable_rails import RunnableRails
guarded_chain = RunnableRails(rails) | your_langgraph_app
```

---

### 5. CrewAI Guardrails

**Task-level guardrails** with validation functions.

```python
from crewai import Agent, Task, Crew

# Method 1: Function-based guardrail
def validate_no_pii(output: str) -> tuple[bool, str]:
    """Return (passed, error_message)."""
    import re

    # Check for SSN
    if re.search(r'\b\d{3}-\d{2}-\d{4}\b', output):
        return False, "Output contains SSN - please remove"

    # Check for credit card
    if re.search(r'\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b', output):
        return False, "Output contains credit card number"

    # Check for email in sensitive context
    if "password" in output.lower() and "@" in output:
        return False, "Output may contain credentials"

    return True, ""

def validate_word_count(output: str) -> tuple[bool, str]:
    """Ensure response is concise."""
    word_count = len(output.split())
    if word_count > 500:
        return False, f"Response too long ({word_count} words). Limit to 500."
    return True, ""

def validate_no_competitors(output: str) -> tuple[bool, str]:
    """Block competitor mentions."""
    competitors = ["acme", "globex", "initech"]
    for comp in competitors:
        if comp.lower() in output.lower():
            return False, f"Do not mention competitor: {comp}"
    return True, ""

# Apply to task
support_task = Task(
    description="Respond to customer inquiry: {query}",
    expected_output="Helpful, accurate response",
    agent=support_agent,
    guardrail=[  # Multiple guardrails
        validate_no_pii,
        validate_word_count,
        validate_no_competitors,
    ],
    guardrail_max_retries=3,  # Retry if guardrail fails
)

# Method 2: LLM-as-Judge guardrail
def llm_safety_check(output: str) -> tuple[bool, str]:
    """Use LLM to evaluate safety."""
    from openai import OpenAI
    client = OpenAI()

    response = client.chat.completions.create(
        model="gpt-4o-mini",
        messages=[{
            "role": "system",
            "content": """Evaluate if this customer support response is safe:
            - No harmful content
            - No PII exposure
            - Professional tone
            - Accurate information

            Respond: PASS or FAIL: <reason>"""
        }, {
            "role": "user",
            "content": output
        }]
    )

    result = response.choices[0].message.content
    if result.startswith("PASS"):
        return True, ""
    return False, result

# Apply
task = Task(
    description="...",
    guardrail=llm_safety_check,
    guardrail_max_retries=2
)
```

---

### Guardrails Feature Comparison

| Feature | AWS Bedrock | Google ADK | Agno | LangGraph | CrewAI |
|---------|-------------|------------|------|-----------|--------|
| **Content Filtering** | ✅ Native | ✅ Gemini | ✅ OpenAI Mod | ⚠️ External | ⚠️ Custom |
| **PII Detection** | ✅ + Masking | ⚠️ Custom | ✅ Built-in | ⚠️ External | ⚠️ Custom |
| **Prompt Injection** | ✅ Native | ⚠️ Callback | ✅ Built-in | ⚠️ NeMo | ⚠️ Custom |
| **Topic Blocking** | ✅ Native | ⚠️ Custom | ⚠️ Custom | ⚠️ External | ⚠️ Custom |
| **Hallucination Check** | ✅ Grounding | ⚠️ Plugin | ⚠️ Custom | ⚠️ External | ✅ Enterprise |
| **No-Code Config** | ✅ Console | ❌ | ❌ | ❌ | ❌ |
| **LLM-as-Judge** | ⚠️ Custom | ✅ Plugin | ⚠️ Custom | ⚠️ Custom | ⚠️ Custom |
| **Retry on Fail** | ✅ | ⚠️ Custom | ⚠️ Custom | ⚠️ Custom | ✅ Native |

---

### External Guardrails Libraries

All frameworks can integrate with these external libraries:

| Library | Best For | Integration Effort |
|---------|----------|-------------------|
| [NVIDIA NeMo Guardrails](https://github.com/NVIDIA/NeMo-Guardrails) | Enterprise, content moderation | Medium |
| [Guardrails AI](https://github.com/guardrails-ai/guardrails) | Structured output validation | Low |
| [LlamaGuard](https://ai.meta.com/research/publications/llama-guard-llm-based-input-output-safeguard-for-human-ai-conversations/) | Content safety classification | Medium |
| [Rebuff](https://github.com/protectai/rebuff) | Prompt injection detection | Low |
| [LangKit](https://github.com/whylabs/langkit) | Monitoring & observability | Medium |

---

### Customer Support Guardrails Example

For your real-time customer support agent:

```python
# Recommended: Combine framework guardrails with external libraries

# 1. Input guardrails (before LLM)
input_checks = [
    PIIDetectionGuardrail(action="mask"),      # Mask SSN, CC in transcript
    PromptInjectionGuardrail(),                 # Block jailbreak attempts
    TopicBlocker(["legal advice", "medical"])   # Redirect to specialists
]

# 2. Output guardrails (after LLM)
output_checks = [
    ContentModerationGuardrail(),               # Block harmful content
    CompetitorMentionBlocker(),                 # Don't mention competitors
    HallucinationChecker(knowledge_base),       # Verify facts
    ToneValidator(style="professional"),        # Ensure professional tone
    ResponseLengthValidator(max_words=200)      # Keep responses concise
]

# 3. Real-time specific
realtime_checks = [
    LatencyGuard(max_ms=500),                   # Fail fast if too slow
    SentimentMonitor(alert_threshold=-0.5),     # Alert on negative sentiment
    EscalationTrigger(keywords=["supervisor"])  # Auto-escalate
]
```

---

## When You NEED LangGraph

1. **Iterative refinement** - Content that improves through critique cycles
2. **Debugging production issues** - Replay exact failure scenarios
3. **Compliance/audit trails** - Full execution history with state at each step
4. **Complex approval workflows** - Multiple human checkpoints with state modification
5. **Long-running pipelines** - Must resume from failure, not restart
6. **Research agents** - Explore multiple paths, backtrack, try alternatives

## When Other Frameworks Are Better

- **CrewAI**: Quick prototyping, role-based collaboration, content generation
- **AWS Bedrock**: Managed scaling, AWS integration, compliance requirements
- **Google ADK**: GCP deployment, streaming audio/video, Gemini optimization
- **Agno**: High-performance, real-time systems, memory-constrained environments

---

## Performance Benchmarks (2025)

*Measured on Apple M4 MacBook Pro, October 2025*

### Instantiation Speed (relative)

| Framework | Speed |
|-----------|-------|
| Agno | 1x (baseline) |
| PydanticAI | ~9x slower |
| CrewAI | ~70x slower |
| LangGraph | ~529x slower |

### Memory Usage (relative)

| Framework | Memory |
|-----------|--------|
| Agno | 1x (baseline) |
| PydanticAI | ~4x more |
| CrewAI | ~10x more |
| LangGraph | ~24x more |

### Latency & Token Efficiency

- **LangGraph**: Lowest latency and token usage in complex workflow benchmarks
- **Agno**: Fastest raw execution for high-volume operations
- **CrewAI**: Optimized for collaborative workflows, not raw speed

---

## Deployment Strategy Comparison

### Local Development

| Framework | CLI | Dev UI | Hot Reload |
|-----------|-----|--------|------------|
| Google ADK | ✅ | ✅ | ✅ |
| AWS Bedrock | ⚠️ AWS CLI | ✅ Console | ❌ |
| Agno | ✅ | ⚠️ | ✅ |
| LangGraph | ✅ | ✅ Studio v2 | ✅ |
| CrewAI | ✅ | ⚠️ | ✅ |
| AIPack | ✅ | ❌ | ✅ |

### Cloud Deployment

| Framework | Managed Platform | Self-Hosted | Kubernetes |
|-----------|------------------|-------------|------------|
| Google ADK | Vertex AI Agent Engine | ✅ Cloud Run | ✅ |
| AWS Bedrock | ✅ Bedrock + AgentCore | ❌ | ⚠️ EKS only |
| Agno | AgentOS | ✅ | ✅ |
| LangGraph | LangGraph Platform | ✅ | ✅ |
| CrewAI | CrewAI Enterprise | ✅ | ✅ |
| AIPack | ❌ | ✅ | ⚠️ |

### Vendor Lock-in Risk

| Framework | Lock-in Level | Notes |
|-----------|---------------|-------|
| Google ADK | Medium | Optimized for GCP, but model-agnostic |
| AWS Bedrock | High | Fully managed, AWS-only deployment |
| Agno | Low | Cloud-agnostic, runs in your infrastructure |
| LangGraph | Low-Medium | LangSmith adds dependency, but optional |
| CrewAI | Low | Fully independent framework |
| AIPack | Very Low | Completely portable |

---

## Technology Stack Comparison

### Languages & SDKs

| Framework | Python | TypeScript | Go | Rust |
|-----------|--------|------------|-----|------|
| Google ADK | ✅ | ✅ | ✅ | ❌ |
| AWS Bedrock | ✅ boto3 | ✅ SDK | ✅ SDK | ✅ SDK |
| Agno | ✅ | ❌ | ❌ | ❌ |
| LangGraph | ✅ | ✅ | ❌ | ❌ |
| CrewAI | ✅ | ❌ | ❌ | ❌ |
| AIPack | Lua | ❌ | ❌ | Runtime |

### Model Provider Support

All frameworks support major providers (OpenAI, Anthropic, Google, etc.) through various integration methods:

- **Google ADK**: Native Gemini + LiteLLM for 50+ providers
- **AWS Bedrock**: Claude, Llama, Mistral, Titan, Cohere, AI21 (via Bedrock marketplace)
- **Agno**: 23+ unified provider interfaces
- **LangGraph**: LangChain integrations (most extensive)
- **CrewAI**: Major providers via built-in or custom
- **AIPack**: All major providers

---

## Decision Framework

### Choose Google ADK if:

- ✅ You're deploying on Google Cloud
- ✅ You need enterprise security and compliance
- ✅ Bidirectional audio/video streaming is required
- ✅ You want tight Vertex AI integration

### Choose AWS Bedrock Agents if:

- ✅ You're committed to AWS infrastructure
- ✅ You need managed scaling to thousands of users
- ✅ Compliance requirements are critical (HIPAA, SOC2, FedRAMP)
- ✅ You want to use open-source frameworks with managed infrastructure (AgentCore)
- ✅ Native AWS service integration is important (Lambda, S3, DynamoDB)

### Choose Agno if:

- ✅ Performance is critical (high-volume, real-time)
- ✅ Memory efficiency matters
- ✅ You want multimodal support out of the box
- ✅ You prefer clean, composable code

### Choose LangGraph if:

- ✅ You need complex, stateful workflows
- ✅ Observability and debugging are priorities
- ✅ You want the most mature ecosystem
- ✅ Human-in-the-loop is important

### Choose CrewAI if:

- ✅ You're prototyping quickly
- ✅ Role-based agent collaboration fits your use case
- ✅ You want an intuitive, low-code experience
- ✅ Content creation or research is your focus

### Choose Vercel AI SDK if:

- ✅ Building Next.js / React applications
- ✅ TypeScript-first team
- ✅ Need excellent streaming UX
- ✅ Simple to medium agent complexity
- ✅ Edge deployment is important
- ✅ Already using Vercel platform

### Choose TanStack AI if:

- ✅ Multi-framework project (React + SolidJS + vanilla)
- ✅ Want to avoid Vercel lock-in
- ✅ Need isomorphic tools (same code client/server)
- ✅ Already using TanStack Query/Router
- ✅ Prefer community-driven open source
- ⚠️ Comfortable with alpha software

### Choose AIPack if:

- ✅ You need lightweight, portable agents
- ✅ Resource constraints are a concern
- ✅ You want shareable agent packages
- ✅ Simple workflows are sufficient

---

## TypeScript SDKs: Vercel AI SDK vs TanStack AI

For TypeScript/JavaScript teams, these are your main options:

| Aspect | Vercel AI SDK | TanStack AI |
|--------|---------------|-------------|
| **Maturity** | v5 (production-ready) | Alpha (Dec 2025) |
| **Framework Focus** | React/Next.js optimized | Truly framework-agnostic |
| **Streaming** | Excellent, edge-native | Good, SSE/HTTP adapters |
| **Tool Definition** | Zod schemas | Isomorphic (client+server) |
| **Human-in-Loop** | Manual implementation | Built-in approval workflows |
| **State Management** | BYO (React state, etc.) | Headless, adapter-based |
| **Deployment** | Vercel optimized | Any platform |
| **Multi-Agent** | ❌ No | ❌ No |
| **Community** | Large (Vercel ecosystem) | Growing (TanStack trust) |

### When to Use Each

```
┌─────────────────────────────────────────────────────────────┐
│                    DECISION TREE                            │
└─────────────────────────────────────────────────────────────┘

Building a Next.js app?
  └─► YES → Vercel AI SDK (best integration)
  └─► NO → Continue...

Need multi-agent orchestration?
  └─► YES → Use Python frameworks (LangGraph, CrewAI, Agno)
  └─► NO → Continue...

Using multiple frontend frameworks?
  └─► YES → TanStack AI (React + SolidJS + vanilla)
  └─► NO → Continue...

Need production stability NOW?
  └─► YES → Vercel AI SDK (mature, battle-tested)
  └─► NO → TanStack AI (alpha but promising)

Want to avoid Vercel lock-in?
  └─► YES → TanStack AI
  └─► NO → Vercel AI SDK
```

### Combining with Python Frameworks

Both can be used as **frontend layers** for Python agent backends:

```typescript
// Vercel AI SDK calling LangGraph backend
import { streamText } from 'ai';

const result = await streamText({
  model: customProvider({
    // Proxy to your LangGraph/CrewAI backend
    baseURL: 'https://your-agent-api.com/v1',
  }),
  prompt: userMessage,
});

// Or use Server Actions to call Python agents
async function askAgent(query: string) {
  'use server';
  const response = await fetch('http://langgraph-service:8000/invoke', {
    method: 'POST',
    body: JSON.stringify({ query }),
  });
  return response.json();
}
```

---

## Recommended Approach

A practical strategy used by many teams:

1. **Prototype** in CrewAI (fastest to get started)
2. **Productionize** with one of:
   - LangGraph (most battle-tested, cloud-agnostic)
   - Google ADK / Vertex AI (if on GCP)
   - AWS Bedrock AgentCore (if on AWS) — works with any framework
3. **Optimize** with Agno if performance becomes critical

### Hybrid Strategy (Recommended for Enterprise)

The most resilient strategy is **hybrid**:
- Prototype with open-source frameworks (LangGraph, CrewAI, Agno)
- Deploy production workloads on managed cloud services (Bedrock AgentCore, Vertex AI)
- This avoids lock-in at the framework level while gaining enterprise infrastructure benefits

---

## Sources

### Google ADK
- [Google ADK Documentation](https://google.github.io/adk-docs/)
- [Google ADK GitHub](https://github.com/google/adk-python)
- [Google Developers Blog - ADK Introduction](https://developers.googleblog.com/en/agent-development-kit-easy-to-build-multi-agent-applications/)

### AWS Bedrock Agents
- [AWS Bedrock Agents Documentation](https://docs.aws.amazon.com/bedrock/latest/userguide/agents.html)
- [AWS Blog - Multi-Agent Collaboration](https://aws.amazon.com/blogs/aws/introducing-multi-agent-collaboration-capability-for-amazon-bedrock/)
- [AWS Blog - Bedrock AgentCore GA](https://aws.amazon.com/blogs/machine-learning/amazon-bedrock-agentcore-is-now-generally-available/)
- [AWS Blog - AgentCore Introduction](https://aws.amazon.com/blogs/aws/introducing-amazon-bedrock-agentcore-securely-deploy-and-operate-ai-agents-at-any-scale/)
- [AWS Prescriptive Guidance - Comparing Agentic AI Frameworks](https://docs.aws.amazon.com/prescriptive-guidance/latest/agentic-ai-frameworks/comparing-agentic-ai-frameworks.html)
- [AWS Blog - LangGraph with Bedrock](https://aws.amazon.com/blogs/machine-learning/build-multi-agent-systems-with-langgraph-and-amazon-bedrock/)

### Agno
- [Agno GitHub](https://github.com/agno-agi/agno)
- [Agno Website](https://www.agno.com)

### LangGraph
- [LangGraph Website](https://www.langchain.com/langgraph)
- [LangChain Blog - v1.0 Release](https://blog.langchain.com/langchain-langgraph-1dot0/)

### CrewAI
- [CrewAI Website](https://www.crewai.com/)
- [CrewAI GitHub](https://github.com/crewAIInc/crewAI)

### Vercel AI SDK
- [Vercel AI SDK Docs](https://ai-sdk.dev/docs/introduction)
- [Vercel AI SDK GitHub](https://github.com/vercel/ai)
- [AI SDK 5 Release](https://vercel.com/blog/ai-sdk-5)
- [How to Build AI Agents with Vercel](https://vercel.com/guides/how-to-build-ai-agents-with-vercel-and-the-ai-sdk)

### TanStack AI
- [TanStack AI Docs](https://tanstack.com/ai/latest/docs)
- [TanStack AI GitHub](https://github.com/TanStack/ai)
- [TanStack AI Overview](https://tanstack.com/ai/latest)

### AIPack
- [AIPack Website](https://aipack.ai/)
- [AIPack Introduction](https://news.aipack.ai/p/aipack-introduction)

### Comparison Articles
- [LangWatch - Best AI Agent Frameworks 2025](https://langwatch.ai/blog/best-ai-agent-frameworks-in-2025-comparing-langgraph-dspy-crewai-agno-and-more)
- [Atla AI - Comparing AI Agent Frameworks](https://www.atla-ai.com/post/ai-agent-frameworks)
- [ZenML - Agno vs LangGraph](https://www.zenml.io/blog/agno-vs-langgraph)
- [DataCamp - CrewAI vs LangGraph vs AutoGen](https://www.datacamp.com/tutorial/crewai-vs-langgraph-vs-autogen)
- [Turing - Top 6 AI Agent Frameworks 2025](https://www.turing.com/resources/ai-agent-frameworks)
- [Softcery - 14 AI Agent Frameworks Compared](https://softcery.com/lab/top-14-ai-agent-frameworks-of-2025-a-founders-guide-to-building-smarter-systems)
- [n8n - AI Agent Orchestration Frameworks](https://blog.n8n.io/ai-agent-orchestration-frameworks/)

---

*Last updated: December 2025*

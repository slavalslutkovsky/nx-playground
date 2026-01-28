# ADR-0003: LangGraph for AI Agent Orchestration

## Status

Accepted

## Date

2024-03-01

## Context

We're building AI agents that need to:
- Maintain conversation state across turns
- Use tools (search, calculations, API calls)
- Support complex multi-step workflows
- Allow human-in-the-loop interactions
- Be debuggable and observable

## Decision

Use **LangGraph** as the primary framework for AI agent orchestration.

### Implementation

- Agent graphs in `apps/agents/*/src/graph.ts`
- Shared utilities in `libs/agents/core`
- LangGraph Studio for local development
- LangSmith/Braintrust for production tracing

### Agent Architecture

```typescript
const workflow = new StateGraph(StateAnnotation)
  .addNode('analyze', analyzeNode)
  .addNode('execute', executeNode)
  .addNode('respond', respondNode)
  .addConditionalEdges('analyze', routeDecision)
  .addEdge('execute', 'respond')
  .compile();
```

## Consequences

### Positive

- **State Management**: Built-in persistence and checkpointing
- **Debugging**: LangGraph Studio for visual debugging
- **Flexibility**: Custom nodes, conditional edges, cycles
- **Ecosystem**: Integrates with LangChain tools and models
- **Streaming**: Native streaming support for UI updates
- **Human-in-Loop**: Built-in interrupt/resume patterns

### Negative

- **Complexity**: More setup than simple chains
- **TypeScript Focus**: Less mature Python support for some features
- **Learning Curve**: Graph-based thinking required
- **Vendor Alignment**: Tied to LangChain ecosystem

### Risks

- **Breaking Changes**: LangGraph is rapidly evolving
  - Mitigation: Pin versions, test upgrades carefully
- **Performance**: Graph overhead for simple tasks
  - Mitigation: Use simple chains for straightforward flows

## Alternatives Considered

### LangChain Chains

- Simpler for linear flows
- Rejected: No cycles, limited state management

### AutoGen (Microsoft)

- Good multi-agent conversations
- Rejected: Less flexible state management, Python-focused

### CrewAI

- Easy multi-agent setup
- Rejected: Less control over execution flow

### Custom Implementation

- Full control
- Rejected: Reinventing tested patterns, no tooling

## References

- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/)
- [LangGraph Studio](https://github.com/langchain-ai/langgraph-studio)
- [docs/AGENTS.md](../AGENTS.md)

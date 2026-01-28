# ADR-0005: Braintrust for Agent Tracing

## Status

Accepted

## Date

2024-04-15

## Context

AI agents need comprehensive observability for:
- Debugging agent behavior
- Tracking LLM costs
- Evaluating response quality
- Identifying performance bottlenecks
- A/B testing prompt changes

Requirements:
- Capture input/output for all LLM calls
- Track token usage and estimated costs
- Support custom metrics and scores
- Integrate with existing OpenTelemetry setup
- Provide actionable dashboards

## Decision

Use **Braintrust** as the primary tracing and evaluation platform for AI agents.

### Implementation

- Gateway integration: `apps/agents/gateway/src/middleware/tracing.ts`
- OpenTelemetry export: `k8s/observability/base/opentelemetry-collector.yaml`
- Agent-level tracing via Braintrust SDK

### Integration Points

```typescript
// Gateway middleware
import { initLogger } from 'braintrust';

const logger = initLogger({
  projectName: 'agent-gateway',
  apiKey: process.env.BRAINTRUST_API_KEY,
});

await logger.log({
  input: { messages, config },
  output: { response },
  metadata: { agentName, latencyMs },
  scores: { success: 1, latency: 0.95 },
});
```

## Consequences

### Positive

- **LLM-Specific**: Built for AI observability
- **Evaluations**: A/B test prompts with statistical rigor
- **Cost Tracking**: Token usage and cost estimates
- **Quality Scoring**: Built-in evaluation metrics
- **Dataset Management**: Capture examples for fine-tuning

### Negative

- **Additional Cost**: Per-log pricing
- **Vendor Lock-in**: Proprietary format
- **Learning Curve**: New dashboard to learn

### Risks

- **Data Privacy**: LLM inputs/outputs stored externally
  - Mitigation: Review data retention policies, redact PII
- **Cost Overrun**: High log volume = high costs
  - Mitigation: Sample non-critical logs, set budgets

## Alternatives Considered

### LangSmith

- LangChain's native tracing
- Rejected: Less evaluation features, tied to LangChain

### Weights & Biases

- Good ML experiment tracking
- Rejected: Less LLM-specific features

### Custom OpenTelemetry

- Full control, no vendor lock-in
- Rejected: Would need to build evaluation, dashboards

### Helicone

- Simple LLM proxy
- Rejected: Less evaluation capabilities

## Configuration

```bash
# Required environment variables
BRAINTRUST_API_KEY=sk-...
BRAINTRUST_PROJECT=agent-gateway

# Optional: OpenTelemetry export
OTEL_EXPORTER_OTLP_ENDPOINT=https://api.braintrust.dev/otel
```

## References

- [Braintrust Documentation](https://www.braintrust.dev/docs)
- [Braintrust Logging SDK](https://www.braintrust.dev/docs/guides/logging)
- [apps/agents/gateway/src/middleware/tracing.ts](../../apps/agents/gateway/src/middleware/tracing.ts)

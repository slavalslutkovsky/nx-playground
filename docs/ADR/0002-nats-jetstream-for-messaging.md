# ADR-0002: NATS JetStream for Async Messaging

## Status

Accepted

## Date

2024-02-01

## Context

We need an async messaging system for:
- Background job processing (email sending, notifications)
- Event-driven workflows
- Decoupling producers from consumers
- Reliable message delivery with retries

Requirements:
- At-least-once delivery
- Consumer groups for load balancing
- Dead letter queues for failed messages
- Reasonable operational complexity

## Decision

Use **NATS JetStream** for asynchronous messaging and job processing.

### Implementation

- Worker framework: `libs/core/nats-worker`
- Message traits: `libs/core/messaging` (Job, Processor)
- First implementation: Email worker (`apps/zerg/email-nats`)

### Stream Configuration

```
Stream: EMAIL_JOBS
- Retention: WorkQueue (removed after ack)
- Max Delivery: 3 attempts
- DLQ: EMAIL_DLQ
```

## Consequences

### Positive

- **Simplicity**: Single binary, easy to operate
- **Performance**: Very low latency (sub-millisecond)
- **Consumer Groups**: Built-in load balancing
- **Pull-Based**: Consumers control flow (backpressure)
- **JetStream**: Durable streams with replay capability

### Negative

- **Less Ecosystem**: Fewer connectors than Kafka
- **Smaller Community**: Less Stack Overflow coverage
- **No Complex Routing**: No topic-based routing like RabbitMQ

### Risks

- **Message Loss**: If NATS server fails before persistence
  - Mitigation: JetStream with synchronous acknowledgment
- **Ordering**: Only guaranteed within same subject
  - Mitigation: Design for idempotency

## Alternatives Considered

### Redis Streams

- Already using Redis for caching
- Rejected: Less mature for production messaging, limited consumer group features
- Note: `libs/core/stream-worker` exists as alternative implementation

### RabbitMQ

- More mature, better routing
- Rejected: More operational complexity, not needed for our use cases

### Kafka

- Best for high-throughput event streaming
- Rejected: Overkill for job queues, high operational overhead

### AWS SQS

- Fully managed
- Rejected: Vendor lock-in, higher latency, less flexible

## References

- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [docs/messaging-patterns.md](../messaging-patterns.md)

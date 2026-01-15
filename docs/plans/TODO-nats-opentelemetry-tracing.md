# TODO: OpenTelemetry Tracing for NATS Worker

## Problem

Jaeger traces break at async NATS boundaries. When a request flows through:

```
HTTP Request → gRPC Call → NATS Publish → NATS Consumer → gRPC Call
```

The NATS segment is invisible in Jaeger because trace context isn't propagated through NATS message headers.

## Current State

| Component | OpenTelemetry Support |
|-----------|----------------------|
| `libs/core/grpc` | Optional feature (exists) |
| `libs/core/nats-worker` | None (gap) |

## Implementation Tasks

### 1. Add Dependencies

**File:** `libs/core/nats-worker/Cargo.toml`

```toml
[features]
default = []
opentelemetry = ["dep:opentelemetry", "dep:tracing-opentelemetry"]

[dependencies]
# OpenTelemetry (optional)
opentelemetry = { version = "0.28", optional = true }
tracing-opentelemetry = { version = "0.29", optional = true }
```

---

### 2. Create Header Carrier Adapter

**File:** `libs/core/nats-worker/src/tracing.rs` (new file)

```rust
//! OpenTelemetry context propagation for NATS headers.

use async_nats::HeaderMap;
use opentelemetry::propagation::{Extractor, Injector};

/// Carrier for injecting/extracting trace context from NATS headers.
pub struct NatsHeaderCarrier<'a>(pub &'a mut HeaderMap);

impl Injector for NatsHeaderCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key, value.as_str());
    }
}

/// Read-only carrier for extracting trace context.
pub struct NatsHeaderExtractor<'a>(pub &'a HeaderMap);

impl Extractor for NatsHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|(k, _)| k.as_ref()).collect()
    }
}
```

**File:** `libs/core/nats-worker/src/lib.rs` (add module)

```rust
#[cfg(feature = "opentelemetry")]
pub mod tracing;
```

---

### 3. Update Producer to Inject Trace Context

**File:** `libs/core/nats-worker/src/producer.rs`

Add imports:

```rust
#[cfg(feature = "opentelemetry")]
use crate::tracing::NatsHeaderCarrier;
#[cfg(feature = "opentelemetry")]
use opentelemetry::global;
#[cfg(feature = "opentelemetry")]
use tracing_opentelemetry::OpenTelemetrySpanExt;
```

Update `send` method:

```rust
pub async fn send<J: Job>(&self, job: &J) -> Result<u64, NatsError> {
    let job_json = serde_json::to_vec(job)?;

    // Inject trace context into headers when opentelemetry is enabled
    #[cfg(feature = "opentelemetry")]
    let headers = {
        let mut headers = async_nats::HeaderMap::new();
        let cx = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut NatsHeaderCarrier(&mut headers));
        });
        Some(headers)
    };

    #[cfg(not(feature = "opentelemetry"))]
    let headers: Option<async_nats::HeaderMap> = None;

    let ack = match headers {
        Some(h) => {
            self.jetstream
                .publish_with_headers(self.subject.clone(), h, job_json.into())
                .await
                .map_err(|e| NatsError::publish_error(e.to_string()))?
                .await
                .map_err(|e| NatsError::publish_error(e.to_string()))?
        }
        None => {
            self.jetstream
                .publish(self.subject.clone(), job_json.into())
                .await
                .map_err(|e| NatsError::publish_error(e.to_string()))?
                .await
                .map_err(|e| NatsError::publish_error(e.to_string()))?
        }
    };

    debug!(
        stream = %self.stream_name,
        subject = %self.subject,
        sequence = ack.sequence,
        job_id = %job.job_id(),
        "Published job"
    );

    Ok(ack.sequence)
}
```

Apply same pattern to `send_to` method.

---

### 4. Update Consumer/Message to Expose Headers

**File:** `libs/core/nats-worker/src/consumer.rs`

Ensure `NatsMessage` includes headers:

```rust
pub struct NatsMessage<J: Job> {
    pub job: J,
    pub sequence: u64,
    pub delivery_count: u64,
    pub headers: Option<async_nats::HeaderMap>,  // Add this field
    // ... existing fields
}
```

When fetching messages, preserve headers:

```rust
let headers = msg.headers.clone();
// Include in NatsMessage construction
```

---

### 5. Update Worker to Extract Trace Context

**File:** `libs/core/nats-worker/src/worker.rs`

Add imports:

```rust
#[cfg(feature = "opentelemetry")]
use crate::tracing::NatsHeaderExtractor;
#[cfg(feature = "opentelemetry")]
use opentelemetry::global;
#[cfg(feature = "opentelemetry")]
use tracing_opentelemetry::OpenTelemetrySpanExt;
```

Update `process_message`:

```rust
async fn process_message(&self, message: NatsMessage<J>) -> Result<(), NatsError> {
    let job_id = message.job_id();
    let sequence = message.sequence;
    let retry_count = message.job.retry_count();

    // Extract parent trace context from NATS headers
    #[cfg(feature = "opentelemetry")]
    let span = {
        let parent_cx = message
            .headers
            .as_ref()
            .map(|h| {
                global::get_text_map_propagator(|propagator| {
                    propagator.extract(&NatsHeaderExtractor(h))
                })
            })
            .unwrap_or_default();

        tracing::info_span!(
            "nats.process_job",
            job_id = %job_id,
            sequence = sequence,
            otel.kind = "consumer"
        )
        .set_parent(parent_cx)
    };

    #[cfg(not(feature = "opentelemetry"))]
    let span = tracing::info_span!(
        "nats.process_job",
        job_id = %job_id,
        sequence = sequence
    );

    // Process within the span context
    async {
        debug!(
            job_id = %job_id,
            sequence = sequence,
            retry_count = retry_count,
            "Processing job"
        );

        let start = Instant::now();
        let result = self.processor.process(&message.job).await;
        let duration = start.elapsed();

        match result {
            Ok(()) => {
                message.ack().await?;
                self.metrics.job_processed(duration);
                debug!(
                    job_id = %job_id,
                    duration_ms = duration.as_millis(),
                    "Job processed successfully"
                );
            }
            Err(e) => {
                self.handle_error(message, e).await?;
            }
        }

        Ok(())
    }
    .instrument(span)
    .await
}
```

---

### 6. Enable Feature in Apps

**File:** `apps/zerg/email-nats/Cargo.toml`

```toml
[dependencies]
nats-worker = { workspace = true, features = ["opentelemetry"] }
```

---

## Expected Result

After implementation, Jaeger will show complete traces:

```
HTTP Request (api-gateway)
  └── gRPC Call (tasks-service)
       └── NATS Publish (producer)
            └── NATS Process (worker)    <-- Now visible!
                 └── Email Send (smtp)
```

## Testing

1. Enable `opentelemetry` feature in an app
2. Ensure Jaeger collector is configured
3. Send a request that triggers NATS job
4. Verify trace spans appear connected in Jaeger UI

## References

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry)
- [async-nats Headers](https://docs.rs/async-nats/latest/async_nats/struct.HeaderMap.html)

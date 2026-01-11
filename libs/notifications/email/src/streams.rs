//! Stream definitions for email processing
//!
//! Provides stream configurations for both Redis Streams and NATS JetStream.

use nats_worker::StreamConfig as NatsStreamConfig;
use stream_worker::StreamDef;

// =============================================================================
// Redis Streams Configuration
// =============================================================================

/// Email stream configuration for Redis Streams
///
/// Defines the Redis stream names and settings for email processing.
/// Uses the same stream names as the legacy EmailProducer for compatibility.
pub struct EmailStream;

impl StreamDef for EmailStream {
    /// Main job stream (matches legacy EmailProducer)
    const STREAM_NAME: &'static str = "notifications:email:stream";

    /// Consumer group for email workers (matches legacy EmailConsumer)
    const CONSUMER_GROUP: &'static str = "email-workers";

    /// Dead letter queue for failed emails
    const DLQ_STREAM: &'static str = "notifications:email:dlq";

    /// Keep up to 100k messages in the stream
    const MAX_LENGTH: i64 = 100_000;

    /// Poll every 1 second when idle
    const POLL_INTERVAL_MS: u64 = 1000;

    /// Process up to 10 messages per batch
    const BATCH_SIZE: usize = 10;

    /// Claim abandoned messages after 30 seconds
    const CLAIM_TIMEOUT_MS: u64 = 30_000;
}

// =============================================================================
// NATS JetStream Configuration
// =============================================================================

/// Email stream configuration for NATS JetStream
///
/// Defines the NATS stream and consumer settings for email processing.
pub struct EmailNatsStream;

impl NatsStreamConfig for EmailNatsStream {
    /// JetStream stream name
    const STREAM_NAME: &'static str = "EMAILS";

    /// Consumer name for email workers
    const CONSUMER_NAME: &'static str = "email-worker";

    /// Dead letter queue stream
    const DLQ_STREAM: &'static str = "EMAILS_DLQ";

    /// Subject pattern for email jobs
    const SUBJECT: &'static str = "emails.>";

    /// Max delivery attempts before DLQ
    const MAX_DELIVER: i64 = 5;

    /// Ack wait timeout (30 seconds)
    const ACK_WAIT_SECS: u64 = 30;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_stream_def() {
        assert_eq!(EmailStream::STREAM_NAME, "notifications:email:stream");
        assert_eq!(EmailStream::CONSUMER_GROUP, "email-workers");
        assert_eq!(EmailStream::DLQ_STREAM, "notifications:email:dlq");
    }

    #[test]
    fn test_nats_stream_config() {
        assert_eq!(EmailNatsStream::STREAM_NAME, "EMAILS");
        assert_eq!(EmailNatsStream::CONSUMER_NAME, "email-worker");
        assert_eq!(EmailNatsStream::DLQ_STREAM, "EMAILS_DLQ");
        assert_eq!(EmailNatsStream::SUBJECT, "emails.>");
    }
}

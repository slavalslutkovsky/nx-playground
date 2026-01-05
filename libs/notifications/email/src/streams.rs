//! EmailStream - Stream definition for email processing
//!
//! Implements the StreamDef trait to define stream configuration.

use stream_worker::StreamDef;

/// Email stream configuration
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_def() {
        assert_eq!(EmailStream::STREAM_NAME, "notifications:email:stream");
        assert_eq!(EmailStream::CONSUMER_GROUP, "email-workers");
        assert_eq!(EmailStream::DLQ_STREAM, "notifications:email:dlq");
    }
}

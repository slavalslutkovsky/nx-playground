//! Stream definitions for the notifications domain.
//!
//! This module defines the Redis stream configuration for email processing.

use stream_worker::StreamDef;

/// Email jobs stream definition.
///
/// Used by the email-worker to process background email jobs.
pub struct EmailStream;

impl StreamDef for EmailStream {
    /// Stream name for email jobs.
    const STREAM_NAME: &'static str = "email:jobs";

    /// Consumer group for email workers.
    const CONSUMER_GROUP: &'static str = "email_workers";

    /// Dead letter queue for failed email jobs.
    const DLQ_STREAM: &'static str = "email:dlq";

    /// Maximum stream length (100k entries).
    const MAX_LENGTH: i64 = 100_000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_stream_def() {
        assert_eq!(EmailStream::stream_name(), "email:jobs");
        assert_eq!(EmailStream::consumer_group(), "email_workers");
        assert_eq!(EmailStream::dlq_stream(), "email:dlq");
        assert_eq!(EmailStream::MAX_LENGTH, 100_000);
    }
}

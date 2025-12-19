//! Stream definitions for the tasks domain.
//!
//! This module defines Redis stream configuration for task processing.

use stream_worker::StreamDef;

/// Task commands stream definition.
///
/// Used by the tasks-worker to process CRUD operations on tasks
/// via Redis streams.
pub struct TaskCommandStream;

impl StreamDef for TaskCommandStream {
    /// Stream name for task commands.
    const STREAM_NAME: &'static str = "tasks:commands";

    /// Consumer group for task workers.
    const CONSUMER_GROUP: &'static str = "task_workers";

    /// Dead letter queue for failed task commands.
    const DLQ_STREAM: &'static str = "tasks:commands:dlq";

    /// Maximum stream length (100k entries).
    const MAX_LENGTH: i64 = 100_000;
}

/// Task results stream definition.
///
/// Workers write results here for API consumers waiting on responses.
pub struct TaskResultStream;

impl StreamDef for TaskResultStream {
    /// Stream name for task results.
    const STREAM_NAME: &'static str = "tasks:results";

    /// Consumer group for API consumers.
    const CONSUMER_GROUP: &'static str = "api_consumers";

    /// Dead letter queue for unprocessed results.
    const DLQ_STREAM: &'static str = "tasks:results:dlq";

    /// Shorter max length for results (they should be consumed quickly).
    const MAX_LENGTH: i64 = 10_000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_command_stream_def() {
        assert_eq!(TaskCommandStream::stream_name(), "tasks:commands");
        assert_eq!(TaskCommandStream::consumer_group(), "task_workers");
        assert_eq!(TaskCommandStream::dlq_stream(), "tasks:commands:dlq");
    }

    #[test]
    fn test_task_result_stream_def() {
        assert_eq!(TaskResultStream::stream_name(), "tasks:results");
        assert_eq!(TaskResultStream::consumer_group(), "api_consumers");
    }
}

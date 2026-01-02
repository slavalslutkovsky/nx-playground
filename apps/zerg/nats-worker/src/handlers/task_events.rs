//! Task event handler implementation

use super::events::{TaskCreated, TaskDeleted, TaskStatusChanged, TaskUpdated};
use crate::messaging::{MessageBroker, MessageStream, ReceivedMessage};
use eyre::Result;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

/// Handler for task-related events
pub struct TaskEventHandler<B: MessageBroker> {
    broker: Arc<B>,
}

impl<B: MessageBroker> TaskEventHandler<B> {
    pub fn new(broker: Arc<B>) -> Self {
        Self { broker }
    }

    /// Run the event handler, subscribing to all task-related subjects
    pub async fn run(&self, subjects: &[&str]) -> Result<()> {
        // Use queue groups for load balancing across worker instances
        let queue_group = "task-workers";

        // Subscribe to all subjects
        let mut streams: Vec<Box<dyn MessageStream>> = Vec::new();
        for subject in subjects {
            let stream = self.broker.queue_subscribe(subject, queue_group).await?;
            streams.push(stream);
        }

        info!(
            subjects = ?subjects,
            queue_group = %queue_group,
            "Task event handler started"
        );

        // Process messages from all streams
        // In production, you'd use tokio::select! or spawn tasks for each stream
        self.process_streams(streams).await
    }

    async fn process_streams(&self, mut streams: Vec<Box<dyn MessageStream>>) -> Result<()> {
        // Simple round-robin processing for now
        // In production, use tokio::select! or spawn separate tasks
        loop {
            for stream in &mut streams {
                if let Some(msg) = tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    stream.next(),
                )
                .await
                .ok()
                .flatten()
                {
                    if let Err(e) = self.handle_message(msg).await {
                        error!(error = %e, "Failed to handle message");
                    }
                }
            }

            // Yield to prevent busy-waiting
            tokio::task::yield_now().await;
        }
    }

    #[instrument(skip(self, msg), fields(subject = %msg.subject))]
    async fn handle_message(&self, msg: ReceivedMessage) -> Result<()> {
        let subject = &msg.subject;

        match subject.as_str() {
            "tasks.created" => self.handle_task_created(&msg).await,
            "tasks.updated" => self.handle_task_updated(&msg).await,
            "tasks.deleted" => self.handle_task_deleted(&msg).await,
            "tasks.status_changed" => self.handle_status_changed(&msg).await,
            _ => {
                warn!(subject = %subject, "Unknown subject, ignoring message");
                Ok(())
            }
        }
    }

    async fn handle_task_created(&self, msg: &ReceivedMessage) -> Result<()> {
        let event: TaskCreated = msg.parse_payload()?;

        info!(
            task_id = %event.id,
            title = %event.title,
            status = %event.status,
            "Task created event received"
        );

        // TODO: Implement business logic
        // - Send notifications
        // - Update search index
        // - Trigger workflows
        // - Publish to other services

        // Example: Publish notification event
        // let notification = EventEnvelope::new(
        //     "notifications.task_created",
        //     "zerg-nats-worker",
        //     NotificationPayload { ... },
        // );
        // self.broker.publish("notifications.send", &notification).await?;

        Ok(())
    }

    async fn handle_task_updated(&self, msg: &ReceivedMessage) -> Result<()> {
        let event: TaskUpdated = msg.parse_payload()?;

        info!(
            task_id = %event.id,
            title = ?event.title,
            status = ?event.status,
            "Task updated event received"
        );

        // TODO: Implement business logic
        // - Update search index
        // - Notify watchers
        // - Audit logging

        Ok(())
    }

    async fn handle_task_deleted(&self, msg: &ReceivedMessage) -> Result<()> {
        let event: TaskDeleted = msg.parse_payload()?;

        info!(
            task_id = %event.id,
            deleted_by = ?event.deleted_by,
            "Task deleted event received"
        );

        // TODO: Implement business logic
        // - Remove from search index
        // - Clean up related resources
        // - Notify subscribers

        Ok(())
    }

    async fn handle_status_changed(&self, msg: &ReceivedMessage) -> Result<()> {
        let event: TaskStatusChanged = msg.parse_payload()?;

        info!(
            task_id = %event.id,
            old_status = %event.old_status,
            new_status = %event.new_status,
            "Task status changed event received"
        );

        // TODO: Implement business logic
        // - Trigger status-based workflows
        // - Update metrics/analytics
        // - Send status change notifications

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{EventEnvelope, MessageStream, ReceivedMessage};
    use async_trait::async_trait;
    use serde::Serialize;
    use std::sync::Mutex;

    /// Mock broker for testing
    struct MockBroker {
        published: Mutex<Vec<(String, Vec<u8>)>>,
    }

    impl MockBroker {
        fn new() -> Self {
            Self {
                published: Mutex::new(Vec::new()),
            }
        }
    }

    struct MockStream;

    #[async_trait]
    impl MessageStream for MockStream {
        async fn next(&mut self) -> Option<ReceivedMessage> {
            None
        }
    }

    #[async_trait]
    impl MessageBroker for MockBroker {
        async fn publish<T: Serialize + Send + Sync>(
            &self,
            subject: &str,
            event: &EventEnvelope<T>,
        ) -> Result<()> {
            let payload = serde_json::to_vec(event)?;
            self.published
                .lock()
                .unwrap()
                .push((subject.to_string(), payload));
            Ok(())
        }

        async fn publish_raw(&self, subject: &str, payload: &[u8]) -> Result<()> {
            self.published
                .lock()
                .unwrap()
                .push((subject.to_string(), payload.to_vec()));
            Ok(())
        }

        async fn subscribe(&self, _subject: &str) -> Result<Box<dyn MessageStream>> {
            Ok(Box::new(MockStream))
        }

        async fn request<T: Serialize + Send + Sync, R: serde::de::DeserializeOwned>(
            &self,
            _subject: &str,
            _request: &T,
        ) -> Result<R> {
            unimplemented!()
        }

        async fn queue_subscribe(
            &self,
            _subject: &str,
            _queue_group: &str,
        ) -> Result<Box<dyn MessageStream>> {
            Ok(Box::new(MockStream))
        }
    }

    #[tokio::test]
    async fn test_handle_task_created() {
        let broker = Arc::new(MockBroker::new());
        let handler = TaskEventHandler::new(broker);

        let payload = TaskCreated {
            id: "123".to_string(),
            title: "Test Task".to_string(),
            description: None,
            status: "pending".to_string(),
            priority: "medium".to_string(),
            created_by: None,
        };

        let msg = ReceivedMessage {
            subject: "tasks.created".to_string(),
            payload: serde_json::to_vec(&payload).unwrap(),
            reply: None,
        };

        let result = handler.handle_message(msg).await;
        assert!(result.is_ok());
    }
}

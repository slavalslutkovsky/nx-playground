//! Event publishing module for NATS messaging.
//!
//! Provides event types and publishing functionality for task events.

use async_nats::Client;
use serde::Serialize;
use tracing::{error, info, instrument};

/// NATS event publisher
#[derive(Clone)]
pub struct EventPublisher {
    client: Client,
}

impl EventPublisher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Publish an event to a subject
    #[instrument(skip(self, event), fields(subject = %subject))]
    pub async fn publish<T: Serialize>(&self, subject: &str, event: &T) {
        match serde_json::to_vec(event) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.to_string(), payload.into()).await {
                    error!(error = %e, subject = %subject, "Failed to publish event");
                } else {
                    info!(subject = %subject, "Event published");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to serialize event");
            }
        }
    }

    /// Publish task created event
    pub async fn task_created(&self, event: &TaskCreatedEvent) {
        self.publish("tasks.created", event).await;
    }

    /// Publish task updated event
    pub async fn task_updated(&self, event: &TaskUpdatedEvent) {
        self.publish("tasks.updated", event).await;
    }

    /// Publish task deleted event
    pub async fn task_deleted(&self, event: &TaskDeletedEvent) {
        self.publish("tasks.deleted", event).await;
    }
}

/// Event published when a task is created
#[derive(Debug, Clone, Serialize)]
pub struct TaskCreatedEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
}

/// Event published when a task is updated
#[derive(Debug, Clone, Serialize)]
pub struct TaskUpdatedEvent {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

/// Event published when a task is deleted
#[derive(Debug, Clone, Serialize)]
pub struct TaskDeletedEvent {
    pub id: String,
}

impl From<&domain_tasks::Task> for TaskCreatedEvent {
    fn from(task: &domain_tasks::Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            status: format!("{:?}", task.status),
            priority: format!("{:?}", task.priority),
        }
    }
}

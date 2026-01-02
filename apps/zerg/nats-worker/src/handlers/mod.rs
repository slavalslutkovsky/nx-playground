//! Event handlers for processing NATS messages

mod task_events;

pub use task_events::TaskEventHandler;

use crate::messaging::{EventEnvelope, ReceivedMessage};
use async_trait::async_trait;
use eyre::Result;

/// Trait for event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, message: ReceivedMessage) -> Result<()>;
}

/// Common event types used across handlers
pub mod events {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskCreated {
        pub id: String,
        pub title: String,
        pub description: Option<String>,
        pub status: String,
        pub priority: String,
        pub created_by: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskUpdated {
        pub id: String,
        pub title: Option<String>,
        pub description: Option<String>,
        pub status: Option<String>,
        pub priority: Option<String>,
        pub updated_by: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskDeleted {
        pub id: String,
        pub deleted_by: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskStatusChanged {
        pub id: String,
        pub old_status: String,
        pub new_status: String,
        pub changed_by: Option<String>,
    }
}

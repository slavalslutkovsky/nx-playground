//! Stream models for task processing.
//!
//! This module defines the command and result models used for
//! processing tasks via Redis streams.

use crate::models::{CreateTask, Task, TaskFilter, UpdateTask};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stream_worker::{Action, StreamJob};
use uuid::Uuid;

/// A command to be processed by the tasks worker.
///
/// This is the job type that flows through the tasks:commands stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommand {
    /// Unique command ID.
    pub id: Uuid,
    /// The action to perform.
    pub action: Action,
    /// Command payload.
    pub payload: TaskCommandPayload,
    /// Correlation ID for request/response matching.
    pub correlation_id: String,
    /// Current retry count.
    pub retry_count: u32,
    /// Command creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl TaskCommand {
    /// Create a new task command.
    pub fn new(action: Action, payload: TaskCommandPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            payload,
            correlation_id: Uuid::new_v4().to_string(),
            retry_count: 0,
            created_at: Utc::now(),
        }
    }

    /// Create a command with a specific correlation ID.
    pub fn with_correlation_id(
        action: Action,
        payload: TaskCommandPayload,
        correlation_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            payload,
            correlation_id: correlation_id.into(),
            retry_count: 0,
            created_at: Utc::now(),
        }
    }
}

impl StreamJob for TaskCommand {
    fn job_id(&self) -> String {
        self.id.to_string()
    }

    fn retry_count(&self) -> u32 {
        self.retry_count
    }

    fn with_retry(&self) -> Self {
        Self {
            id: self.id, // Keep same ID for retries
            retry_count: self.retry_count + 1,
            ..self.clone()
        }
    }

    fn max_retries(&self) -> u32 {
        3
    }
}

/// Payload for task commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskCommandPayload {
    /// Create a new task.
    Create(CreateTask),
    /// Get a task by ID.
    GetById { task_id: Uuid },
    /// Update an existing task.
    Update { task_id: Uuid, data: UpdateTask },
    /// Delete a task.
    Delete { task_id: Uuid },
    /// List tasks with filters.
    List(TaskFilter),
    /// Mark a task as completed.
    Complete { task_id: Uuid },
    /// Mark a task as incomplete.
    Uncomplete { task_id: Uuid },
}

/// Result of a task command.
///
/// This is sent back to the tasks:results stream (or stored for polling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommandResult {
    /// Correlation ID matching the original command.
    pub correlation_id: String,
    /// Whether the command succeeded.
    pub success: bool,
    /// Result data (on success).
    pub data: Option<TaskResultData>,
    /// Error message (on failure).
    pub error: Option<String>,
    /// Processing timestamp.
    pub processed_at: DateTime<Utc>,
}

impl TaskCommandResult {
    /// Create a success result with data.
    pub fn success(correlation_id: impl Into<String>, data: TaskResultData) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            success: true,
            data: Some(data),
            error: None,
            processed_at: Utc::now(),
        }
    }

    /// Create a failure result with error message.
    pub fn failure(correlation_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            success: false,
            data: None,
            error: Some(error.into()),
            processed_at: Utc::now(),
        }
    }
}

/// Data returned from task commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskResultData {
    /// Single task returned.
    Single { task: Task },
    /// List of tasks returned.
    List { tasks: Vec<Task> },
    /// Task was deleted.
    Deleted { id: Uuid },
    /// Count of tasks.
    Count { count: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskPriority;

    #[test]
    fn test_task_command_new() {
        let cmd = TaskCommand::new(
            Action::Create,
            TaskCommandPayload::Create(CreateTask {
                title: "Test Task".to_string(),
                description: String::new(),
                project_id: None,
                priority: TaskPriority::Medium,
                status: crate::models::TaskStatus::Todo,
                due_date: None,
            }),
        );

        assert_eq!(cmd.action, Action::Create);
        assert_eq!(cmd.retry_count, 0);
    }

    #[test]
    fn test_task_command_stream_job() {
        let cmd = TaskCommand::new(
            Action::Delete,
            TaskCommandPayload::Delete {
                task_id: Uuid::new_v4(),
            },
        );

        assert_eq!(cmd.retry_count(), 0);
        assert_eq!(cmd.max_retries(), 3);

        let retry = cmd.with_retry();
        assert_eq!(retry.retry_count(), 1);
        assert_eq!(retry.id, cmd.id); // Same ID for retries
    }

    #[test]
    fn test_task_result_success() {
        let result = TaskCommandResult::success(
            "corr-123",
            TaskResultData::Deleted {
                id: Uuid::new_v4(),
            },
        );

        assert!(result.success);
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let result = TaskCommandResult::failure("corr-123", "Task not found");

        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("Task not found".to_string()));
    }
}

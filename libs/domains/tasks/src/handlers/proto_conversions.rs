//! Proto conversion helpers for gRPC handlers
//!
//! These functions convert between proto types and domain types.
//! Copied from apps/zerg/tasks/src/conversions.rs to avoid cyclic dependency.

use chrono::{DateTime, Utc};
use rpc::tasks::{Priority, Status};
use uuid::Uuid;

use crate::models::{TaskPriority, TaskStatus};

// UUID conversions
pub fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

pub fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, String> {
    Uuid::from_slice(bytes).map_err(|e| format!("Invalid UUID bytes: {}", e))
}

pub fn opt_uuid_to_bytes(uuid: Option<Uuid>) -> Option<Vec<u8>> {
    uuid.map(|u| u.as_bytes().to_vec())
}

pub fn opt_bytes_to_uuid(bytes: Option<Vec<u8>>) -> Result<Option<Uuid>, String> {
    bytes.map(|b| bytes_to_uuid(&b)).transpose()
}

// Priority conversions
pub fn domain_priority_to_proto(priority: &TaskPriority) -> i32 {
    match priority {
        TaskPriority::Low => Priority::Low as i32,
        TaskPriority::Medium => Priority::Medium as i32,
        TaskPriority::High => Priority::High as i32,
        TaskPriority::Urgent => Priority::Urgent as i32,
    }
}

pub fn proto_priority_to_domain(priority: i32) -> Result<TaskPriority, String> {
    match Priority::try_from(priority) {
        Ok(Priority::Low) => Ok(TaskPriority::Low),
        Ok(Priority::Medium) => Ok(TaskPriority::Medium),
        Ok(Priority::High) => Ok(TaskPriority::High),
        Ok(Priority::Urgent) => Ok(TaskPriority::Urgent),
        _ => Err(format!("Invalid priority: {}", priority)),
    }
}

// Status conversions
pub fn domain_status_to_proto(status: &TaskStatus) -> i32 {
    match status {
        TaskStatus::Todo => Status::Todo as i32,
        TaskStatus::InProgress => Status::InProgress as i32,
        TaskStatus::Done => Status::Done as i32,
    }
}

pub fn proto_status_to_domain(status: i32) -> Result<TaskStatus, String> {
    match Status::try_from(status) {
        Ok(Status::Todo) => Ok(TaskStatus::Todo),
        Ok(Status::InProgress) => Ok(TaskStatus::InProgress),
        Ok(Status::Done) => Ok(TaskStatus::Done),
        _ => Err(format!("Invalid status: {}", status)),
    }
}

// Timestamp conversions
pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

pub fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now())
}

pub fn opt_timestamp_to_datetime(timestamp: Option<i64>) -> Option<DateTime<Utc>> {
    timestamp.map(timestamp_to_datetime)
}

pub fn opt_datetime_to_timestamp(dt: Option<DateTime<Utc>>) -> Option<i64> {
    dt.map(datetime_to_timestamp)
}

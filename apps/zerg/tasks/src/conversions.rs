use chrono::{DateTime, Utc};
use domain_tasks::{TaskPriority, TaskStatus};
use rpc::tasks::{Priority, Status};
use tonic::Status as TonicStatus;
use uuid::Uuid;

// UUID conversions
pub fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

pub fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, TonicStatus> {
    Uuid::from_slice(bytes).map_err(|_| TonicStatus::invalid_argument("Invalid UUID bytes"))
}

pub fn opt_uuid_to_bytes(uuid: Option<Uuid>) -> Option<Vec<u8>> {
    uuid.map(|u| u.as_bytes().to_vec())
}

pub fn opt_bytes_to_uuid(bytes: Option<Vec<u8>>) -> Result<Option<Uuid>, TonicStatus> {
    bytes
        .map(|b| bytes_to_uuid(&b))
        .transpose()
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

pub fn proto_priority_to_domain(priority: i32) -> Result<TaskPriority, TonicStatus> {
    match Priority::try_from(priority) {
        Ok(Priority::Low) => Ok(TaskPriority::Low),
        Ok(Priority::Medium) => Ok(TaskPriority::Medium),
        Ok(Priority::High) => Ok(TaskPriority::High),
        Ok(Priority::Urgent) => Ok(TaskPriority::Urgent),
        _ => Err(TonicStatus::invalid_argument(format!(
            "Invalid priority: {}",
            priority
        ))),
    }
}

pub fn opt_proto_priority_to_domain(priority: Option<i32>) -> Result<Option<TaskPriority>, TonicStatus> {
    priority.map(proto_priority_to_domain).transpose()
}

// Status conversions
pub fn domain_status_to_proto(status: &TaskStatus) -> i32 {
    match status {
        TaskStatus::Todo => Status::Todo as i32,
        TaskStatus::InProgress => Status::InProgress as i32,
        TaskStatus::Done => Status::Done as i32,
    }
}

pub fn proto_status_to_domain(status: i32) -> Result<TaskStatus, TonicStatus> {
    match Status::try_from(status) {
        Ok(Status::Todo) => Ok(TaskStatus::Todo),
        Ok(Status::InProgress) => Ok(TaskStatus::InProgress),
        Ok(Status::Done) => Ok(TaskStatus::Done),
        _ => Err(TonicStatus::invalid_argument(format!(
            "Invalid status: {}",
            status
        ))),
    }
}

pub fn opt_proto_status_to_domain(status: Option<i32>) -> Result<Option<TaskStatus>, TonicStatus> {
    status.map(proto_status_to_domain).transpose()
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

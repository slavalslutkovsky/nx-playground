//! Proto conversion helpers
//!
//! Re-exports conversions from the domain_tasks crate with tonic::Status error handling.

use domain_tasks::{TaskPriority, TaskStatus};
use tonic::Status as TonicStatus;
use uuid::Uuid;

// Re-export functions that don't return Results
pub use domain_tasks::conversions::{
    datetime_to_timestamp, domain_priority_to_proto, domain_status_to_proto,
    opt_datetime_to_timestamp, opt_timestamp_to_datetime, opt_uuid_to_bytes, timestamp_to_datetime,
    uuid_to_bytes,
};

// Wrapper functions that convert String errors to TonicStatus for gRPC service

pub fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, TonicStatus> {
    domain_tasks::conversions::bytes_to_uuid(bytes)
        .map_err(|_| TonicStatus::invalid_argument("Invalid UUID bytes"))
}

pub fn opt_bytes_to_uuid(bytes: Option<Vec<u8>>) -> Result<Option<Uuid>, TonicStatus> {
    domain_tasks::conversions::opt_bytes_to_uuid(bytes)
        .map_err(|_| TonicStatus::invalid_argument("Invalid UUID bytes"))
}

pub fn proto_priority_to_domain(priority: i32) -> Result<TaskPriority, TonicStatus> {
    domain_tasks::conversions::proto_priority_to_domain(priority)
        .map_err(|e| TonicStatus::invalid_argument(e))
}

pub fn opt_proto_priority_to_domain(
    priority: Option<i32>,
) -> Result<Option<TaskPriority>, TonicStatus> {
    domain_tasks::conversions::opt_proto_priority_to_domain(priority)
        .map_err(|e| TonicStatus::invalid_argument(e))
}

pub fn proto_status_to_domain(status: i32) -> Result<TaskStatus, TonicStatus> {
    domain_tasks::conversions::proto_status_to_domain(status)
        .map_err(|e| TonicStatus::invalid_argument(e))
}

pub fn opt_proto_status_to_domain(status: Option<i32>) -> Result<Option<TaskStatus>, TonicStatus> {
    domain_tasks::conversions::opt_proto_status_to_domain(status)
        .map_err(|e| TonicStatus::invalid_argument(e))
}

//! Audit logging for security and compliance.
//!
//! This module provides structured audit logging for tracking security-relevant events
//! such as authentication, authorization, and data modifications.
//!
//! # Example
//! ```ignore
//! use axum_helpers::audit::{AuditEvent, AuditOutcome};
//!
//! // Log successful login
//! AuditEvent::new(
//!     Some("user123".to_string()),
//!     "user.login",
//!     None,
//!     AuditOutcome::Success
//! )
//! .with_ip(extract_ip_from_headers(&headers))
//! .with_user_agent(extract_user_agent(&headers))
//! .log();
//!
//! // Log authorization failure
//! AuditEvent::new(
//!     Some("user123".to_string()),
//!     "resource.access",
//!     Some("project:456".to_string()),
//!     AuditOutcome::Denied
//! )
//! .with_details(json!({"reason": "insufficient_permissions"}))
//! .log();
//! ```

use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::SocketAddr;

/// Outcome of an audited action.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    /// Action completed successfully
    Success,
    /// Action failed (e.g., validation error, system error)
    Failure,
    /// Action was denied (e.g., insufficient permissions)
    Denied,
}

/// Structured audit event for security and compliance logging.
///
/// Use the builder pattern to construct audit events with optional fields,
/// then call `.log()` to emit the event to the audit log.
#[derive(Debug, Serialize)]
pub struct AuditEvent {
    /// User who performed the action (if authenticated)
    pub user_id: Option<String>,
    /// Action performed (e.g., "user.login", "project.delete", "resource.access")
    pub action: String,
    /// Resource affected (e.g., "project:123", "user:456")
    pub resource: Option<String>,
    /// Outcome of the action
    pub outcome: AuditOutcome,
    /// Client IP address
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Timestamp when the event occurred
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Additional details about the event (JSON)
    pub details: Option<serde_json::Value>,
}

impl AuditEvent {
    /// Create a new audit event.
    ///
    /// # Arguments
    /// * `user_id` - User who performed the action (None for unauthenticated actions)
    /// * `action` - Action identifier (e.g., "user.login", "project.create")
    /// * `resource` - Resource identifier (e.g., "project:123", "user:456")
    /// * `outcome` - Success, Failure, or Denied
    pub fn new(
        user_id: Option<String>,
        action: impl Into<String>,
        resource: Option<String>,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            user_id,
            action: action.into(),
            resource,
            outcome,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
            details: None,
        }
    }

    /// Add IP address to the audit event.
    pub fn with_ip(mut self, ip: Option<String>) -> Self {
        self.ip_address = ip;
        self
    }

    /// Add user agent to the audit event.
    pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Add additional details to the audit event.
    ///
    /// The details will be serialized to JSON.
    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = serde_json::to_value(details).ok();
        self
    }

    /// Emit the audit event to the audit log.
    ///
    /// This logs to the "audit" target with structured fields.
    /// Configure your logging backend to route audit logs to a separate file/system.
    pub fn log(self) {
        tracing::info!(
            target: "audit",
            user_id = self.user_id,
            action = %self.action,
            resource = self.resource,
            outcome = ?self.outcome,
            ip = self.ip_address,
            user_agent = self.user_agent,
            timestamp = %self.timestamp,
            details = ?self.details,
            "{}",
            serde_json::to_string(&self).unwrap_or_else(|_| "Failed to serialize audit event".to_string())
        );
    }
}

/// Extract client IP address from HTTP headers.
///
/// Checks X-Forwarded-For and X-Real-IP headers to get the real client IP
/// when behind a proxy or load balancer.
///
/// Returns the first IP from X-Forwarded-For (most accurate) or X-Real-IP as fallback.
pub fn extract_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
}

/// Extract client IP address from socket address.
///
/// Use this as a fallback when proxy headers are not available.
pub fn extract_ip_from_socket(socket: Option<SocketAddr>) -> Option<String> {
    socket.map(|addr| addr.ip().to_string())
}

/// Extract user agent string from HTTP headers.
pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

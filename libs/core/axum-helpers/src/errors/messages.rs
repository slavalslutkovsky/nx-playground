//! Standard error messages and codes for consistent error responses.

// Message constants
pub const VALIDATION_FAILED: &str = "Validation failed for the provided input.";
pub const INVALID_UUID: &str = "Invalid UUID format.";
pub const INVALID_JSON: &str = "Invalid JSON format.";
pub const NOT_FOUND_RESOURCE: &str = "Requested resource was not found.";
pub const INTERNAL_ERROR: &str = "An unexpected error occurred.";
pub const DB_ERROR: &str = "A database error occurred.";
pub const DB_CONFIG_ERROR: &str = "Database configuration error.";
pub const DB_IO_ERROR: &str = "Database I/O error.";
pub const DB_TLS_ERROR: &str = "Database TLS error.";
pub const DB_PROTOCOL_ERROR: &str = "Database protocol error.";
pub const DB_TYPE_NOT_FOUND: &str = "Database type not found.";
pub const DB_DECODE_ERROR: &str = "Failed to decode database response.";
pub const DB_ENCODE_ERROR: &str = "Failed to encode database request.";
pub const DB_DRIVER_ERROR: &str = "A database driver error occurred.";
pub const DB_POOL_TIMEOUT: &str = "Database connection pool timed out.";
pub const DB_POOL_CLOSED: &str = "Database connection pool closed.";
pub const DB_WORKER_CRASHED: &str = "Database connection pool worker crashed.";
pub const DB_MIGRATION_ERROR: &str = "Database migration error.";
pub const DB_INTERNAL_ERROR: &str = "Internal database error.";

// Error codes for observability and debugging
pub const CODE_VALIDATION: i32 = 1001;
pub const CODE_UUID: i32 = 1002;
pub const CODE_JSON_EXTRACTION: i32 = 1003;
pub const CODE_NOT_FOUND: i32 = 1004;
pub const CODE_INTERNAL: i32 = 1005;

// Database error codes
pub const CODE_SQLX_NOT_FOUND: i32 = 2001;
pub const CODE_SQLX_CONFIG: i32 = 2002;
pub const CODE_SQLX_DATABASE: i32 = 2003;
pub const CODE_SQLX_IO: i32 = 2004;
pub const CODE_SQLX_TLS: i32 = 2005;
pub const CODE_SQLX_PROTOCOL: i32 = 2006;
pub const CODE_SQLX_TYPE_NOT_FOUND: i32 = 2007;
pub const CODE_SQLX_COLUMN_INDEX: i32 = 2008;
pub const CODE_SQLX_COLUMN_NOT_FOUND: i32 = 2009;
pub const CODE_SQLX_DECODE: i32 = 2010;
pub const CODE_SQLX_ENCODE: i32 = 2011;
pub const CODE_SQLX_DRIVER: i32 = 2012;
pub const CODE_SQLX_POOL_TIMEOUT: i32 = 2013;
pub const CODE_SQLX_POOL_CLOSED: i32 = 2014;
pub const CODE_SQLX_WORKER_CRASHED: i32 = 2015;
pub const CODE_SQLX_MIGRATE: i32 = 2016;
pub const CODE_SQLX_UNHANDLED: i32 = 2099;

// Migration error code
pub const CODE_MIGRATION: i32 = 3001;

// I/O error code
pub const CODE_IO: i32 = 4001;

// JSON parsing error code
pub const CODE_SERDE_JSON: i32 = 5001;

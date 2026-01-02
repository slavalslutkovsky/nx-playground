//! Unified API schema types
//!
//! These types provide an adapter-agnostic representation of API metadata
//! that can be populated from OpenAPI, gRPC, GraphQL, or other sources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};
use utoipa::ToSchema;

/// HTTP method for an endpoint
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

/// Source of an API definition
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiSource {
    /// OpenAPI/Swagger specification
    OpenApi {
        url: Option<String>,
        version: String,
    },
    /// gRPC service
    Grpc { service: String, package: String },
    /// GraphQL endpoint
    GraphQL { endpoint: String },
    /// Internal/embedded API
    #[default]
    Internal,
}

/// Parameter location in the request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// API parameter definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiParameter {
    /// Parameter name
    pub name: String,
    /// Location of the parameter
    pub location: ParameterLocation,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter description
    pub description: Option<String>,
    /// Schema type (e.g., "string", "integer", "uuid")
    pub schema_type: String,
    /// Default value if any
    pub default: Option<serde_json::Value>,
    /// Example value
    pub example: Option<serde_json::Value>,
}

/// Request body definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiRequestBody {
    /// Content type (e.g., "application/json")
    pub content_type: String,
    /// Whether the body is required
    pub required: bool,
    /// Schema name reference
    pub schema_name: Option<String>,
    /// Description of the request body
    pub description: Option<String>,
}

/// Response definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response description
    pub description: String,
    /// Content type if any
    pub content_type: Option<String>,
    /// Schema name reference
    pub schema_name: Option<String>,
}

/// Security requirement for an endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SecurityRequirement {
    /// Security scheme name
    pub scheme: String,
    /// Required scopes
    pub scopes: Vec<String>,
}

/// Unified API endpoint representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiEndpoint {
    /// Full path (e.g., "/api/projects/{id}")
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Operation ID for code generation
    pub operation_id: Option<String>,
    /// Short summary of the endpoint
    pub summary: Option<String>,
    /// Detailed description
    pub description: Option<String>,
    /// Tags for grouping
    pub tags: Vec<String>,
    /// Path, query, header, cookie parameters
    pub parameters: Vec<ApiParameter>,
    /// Request body if any
    pub request_body: Option<ApiRequestBody>,
    /// Possible responses
    pub responses: Vec<ApiResponse>,
    /// Security requirements
    pub security: Vec<SecurityRequirement>,
    /// Source of this endpoint definition
    #[serde(default)]
    pub source: ApiSource,
    /// Whether the endpoint is deprecated
    #[serde(default)]
    pub deprecated: bool,
}

impl ApiEndpoint {
    /// Create a new endpoint with minimal required fields
    pub fn new(path: impl Into<String>, method: HttpMethod) -> Self {
        Self {
            path: path.into(),
            method,
            operation_id: None,
            summary: None,
            description: None,
            tags: Vec::new(),
            parameters: Vec::new(),
            request_body: None,
            responses: Vec::new(),
            security: Vec::new(),
            source: ApiSource::Internal,
            deprecated: false,
        }
    }

    /// Get a unique key for this endpoint
    pub fn key(&self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

/// Schema type enumeration
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    #[default]
    Object,
    Array,
    String,
    Integer,
    Number,
    Boolean,
    Null,
    /// Reference to another schema
    Ref(String),
}

/// Property definition within a schema
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SchemaProperty {
    /// Property name
    pub name: String,
    /// Property type
    pub schema_type: SchemaType,
    /// Whether the property is required
    pub required: bool,
    /// Property description
    pub description: Option<String>,
    /// Format (e.g., "uuid", "date-time", "email")
    pub format: Option<String>,
    /// Enum values if applicable
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub enum_values: Vec<serde_json::Value>,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Example value
    pub example: Option<serde_json::Value>,
    /// Minimum value for numbers
    pub minimum: Option<f64>,
    /// Maximum value for numbers
    pub maximum: Option<f64>,
    /// Min length for strings
    pub min_length: Option<usize>,
    /// Max length for strings
    pub max_length: Option<usize>,
    /// Pattern for strings (regex)
    pub pattern: Option<String>,
}

/// Unified API schema (DTO/model) representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(no_recursion)]
pub struct ApiSchema {
    /// Schema name
    pub name: String,
    /// Schema type
    #[serde(default)]
    pub schema_type: SchemaType,
    /// Properties for object types
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<SchemaProperty>,
    /// Required property names
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required: Vec<String>,
    /// Schema description
    pub description: Option<String>,
    /// Item schema for array types
    #[schema(value_type = Option<Object>)]
    pub items: Option<Box<ApiSchema>>,
    /// Endpoints that use this schema
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub used_in: Vec<String>,
    /// Source of this schema
    #[serde(default)]
    pub source: ApiSource,
    /// Example value
    pub example: Option<serde_json::Value>,
}

impl ApiSchema {
    /// Create a new schema with minimal required fields
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schema_type: SchemaType::Object,
            properties: Vec::new(),
            required: Vec::new(),
            description: None,
            items: None,
            used_in: Vec::new(),
            source: ApiSource::Internal,
            example: None,
        }
    }

    /// Get property names as a vector
    pub fn property_names(&self) -> Vec<&str> {
        self.properties.iter().map(|p| p.name.as_str()).collect()
    }
}

/// Summary statistics for the API
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiStats {
    /// Total number of endpoints
    pub total_endpoints: usize,
    /// Total number of schemas
    pub total_schemas: usize,
    /// Endpoints by HTTP method
    pub endpoints_by_method: HashMap<String, usize>,
    /// Endpoints by tag
    pub endpoints_by_tag: HashMap<String, usize>,
    /// Number of deprecated endpoints
    pub deprecated_endpoints: usize,
    /// Documentation coverage percentage
    pub documentation_coverage: f64,
}

/// Search result for endpoints and schemas
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchResult {
    Endpoint {
        path: String,
        method: String,
        summary: Option<String>,
        tags: Vec<String>,
        #[serde(rename = "match")]
        match_field: String,
    },
    Schema {
        name: String,
        description: Option<String>,
        #[serde(rename = "match")]
        match_field: String,
    },
}

/// Response for listing endpoints
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EndpointListResponse {
    pub total: usize,
    pub endpoints: Vec<ApiEndpoint>,
}

/// Response for listing schemas
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SchemaListResponse {
    pub total: usize,
    pub schemas: Vec<ApiSchema>,
}

/// Response for search queries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub query: String,
    pub total: usize,
    pub results: Vec<SearchResult>,
}

//! OpenAPI 3.x Adapter
//!
//! Parses OpenAPI/Swagger specifications into the unified schema format.

use crate::error::{ApiForgeError, Result};
use crate::registry::ApiRegistry;
use crate::schema::{
    ApiEndpoint, ApiParameter, ApiRequestBody, ApiResponse, ApiSchema, ApiSource, HttpMethod,
    ParameterLocation, SchemaProperty, SchemaType, SecurityRequirement,
};
use async_trait::async_trait;
use openapiv3::{
    OpenAPI, Operation, Parameter as OApiParameter, ParameterSchemaOrContent, PathItem,
    ReferenceOr, Schema, SchemaKind, StatusCode as OApiStatusCode, Type as OApiType,
};

/// Adapter for parsing OpenAPI 3.x specifications
pub struct OpenApiAdapter {
    /// The parsed OpenAPI spec
    spec: OpenAPI,
    /// Source URL or path
    source_url: Option<String>,
}

impl OpenApiAdapter {
    /// Create a new adapter from an OpenAPI spec
    pub fn new(spec: OpenAPI) -> Self {
        Self {
            spec,
            source_url: None,
        }
    }

    /// Create a new adapter with source URL
    pub fn with_source(spec: OpenAPI, url: impl Into<String>) -> Self {
        Self {
            spec,
            source_url: Some(url.into()),
        }
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let spec: OpenAPI =
            serde_json::from_str(json).map_err(|e| ApiForgeError::OpenApiParse(e.to_string()))?;
        Ok(Self::new(spec))
    }

    /// Parse from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let spec: OpenAPI = serde_yaml_ng::from_str(yaml)
            .map_err(|e| ApiForgeError::OpenApiParse(e.to_string()))?;
        Ok(Self::new(spec))
    }

    /// Fetch and parse from URL
    pub async fn from_url(url: &str) -> Result<Self> {
        let response = reqwest::get(url).await?;
        let content = response.text().await?;

        // Try JSON first, then YAML
        let spec = if content.trim().starts_with('{') {
            serde_json::from_str(&content)
                .map_err(|e| ApiForgeError::OpenApiParse(e.to_string()))?
        } else {
            serde_yaml_ng::from_str(&content)
                .map_err(|e| ApiForgeError::OpenApiParse(e.to_string()))?
        };

        Ok(Self::with_source(spec, url))
    }

    /// Get the OpenAPI version
    pub fn version(&self) -> &str {
        &self.spec.openapi
    }

    /// Get the API info
    pub fn info(&self) -> &openapiv3::Info {
        &self.spec.info
    }

    /// Parse all paths into endpoints
    fn parse_endpoints(&self) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        for (path, path_item) in &self.spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                endpoints.extend(self.parse_path_item(path, item));
            }
        }

        endpoints
    }

    /// Parse a single path item into endpoints
    fn parse_path_item(&self, path: &str, item: &PathItem) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // Parse each HTTP method
        if let Some(op) = &item.get {
            endpoints.push(self.parse_operation(path, HttpMethod::Get, op, &item.parameters));
        }
        if let Some(op) = &item.post {
            endpoints.push(self.parse_operation(path, HttpMethod::Post, op, &item.parameters));
        }
        if let Some(op) = &item.put {
            endpoints.push(self.parse_operation(path, HttpMethod::Put, op, &item.parameters));
        }
        if let Some(op) = &item.patch {
            endpoints.push(self.parse_operation(path, HttpMethod::Patch, op, &item.parameters));
        }
        if let Some(op) = &item.delete {
            endpoints.push(self.parse_operation(path, HttpMethod::Delete, op, &item.parameters));
        }
        if let Some(op) = &item.head {
            endpoints.push(self.parse_operation(path, HttpMethod::Head, op, &item.parameters));
        }
        if let Some(op) = &item.options {
            endpoints.push(self.parse_operation(path, HttpMethod::Options, op, &item.parameters));
        }
        if let Some(op) = &item.trace {
            endpoints.push(self.parse_operation(path, HttpMethod::Trace, op, &item.parameters));
        }

        endpoints
    }

    /// Parse a single operation into an endpoint
    fn parse_operation(
        &self,
        path: &str,
        method: HttpMethod,
        op: &Operation,
        path_params: &[ReferenceOr<OApiParameter>],
    ) -> ApiEndpoint {
        // Parse parameters (combine path-level and operation-level)
        let mut parameters: Vec<ApiParameter> = path_params
            .iter()
            .filter_map(|p| self.parse_parameter(p))
            .collect();

        parameters.extend(op.parameters.iter().filter_map(|p| self.parse_parameter(p)));

        // Parse request body
        let request_body = op
            .request_body
            .as_ref()
            .and_then(|rb| self.parse_request_body(rb));

        // Parse responses
        let responses: Vec<ApiResponse> = op
            .responses
            .responses
            .iter()
            .map(|(status, response)| self.parse_response(status, response))
            .collect();

        // Parse security requirements
        let security: Vec<SecurityRequirement> = op
            .security
            .as_ref()
            .map(|sec| {
                sec.iter()
                    .flat_map(|s| {
                        s.iter().map(|(name, scopes)| SecurityRequirement {
                            scheme: name.clone(),
                            scopes: scopes.clone(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        ApiEndpoint {
            path: path.to_string(),
            method,
            operation_id: op.operation_id.clone(),
            summary: op.summary.clone(),
            description: op.description.clone(),
            tags: op.tags.clone(),
            parameters,
            request_body,
            responses,
            security,
            source: ApiSource::OpenApi {
                url: self.source_url.clone(),
                version: self.spec.openapi.clone(),
            },
            deprecated: op.deprecated,
        }
    }

    /// Parse a parameter
    fn parse_parameter(&self, param: &ReferenceOr<OApiParameter>) -> Option<ApiParameter> {
        let param = match param {
            ReferenceOr::Item(p) => p,
            ReferenceOr::Reference { reference } => {
                // Try to resolve reference
                return self.resolve_parameter_ref(reference);
            }
        };

        let (location, data) = match param {
            OApiParameter::Query { parameter_data, .. } => {
                (ParameterLocation::Query, parameter_data)
            }
            OApiParameter::Header { parameter_data, .. } => {
                (ParameterLocation::Header, parameter_data)
            }
            OApiParameter::Path { parameter_data, .. } => (ParameterLocation::Path, parameter_data),
            OApiParameter::Cookie { parameter_data, .. } => {
                (ParameterLocation::Cookie, parameter_data)
            }
        };

        let schema_type = self.get_parameter_schema_type(&data.format);

        Some(ApiParameter {
            name: data.name.clone(),
            location,
            required: data.required,
            description: data.description.clone(),
            schema_type,
            default: None,
            example: data.example.clone(),
        })
    }

    /// Get schema type from parameter format
    fn get_parameter_schema_type(&self, format: &ParameterSchemaOrContent) -> String {
        match format {
            ParameterSchemaOrContent::Schema(schema) => match schema {
                ReferenceOr::Item(s) => self.schema_to_type_string(s),
                ReferenceOr::Reference { reference } => {
                    extract_ref_name(reference).unwrap_or_else(|| "object".to_string())
                }
            },
            ParameterSchemaOrContent::Content(_) => "object".to_string(),
        }
    }

    /// Convert a schema to a type string
    fn schema_to_type_string(&self, schema: &Schema) -> String {
        match &schema.schema_kind {
            SchemaKind::Type(t) => match t {
                OApiType::String(_) => "string".to_string(),
                OApiType::Number(_) => "number".to_string(),
                OApiType::Integer(_) => "integer".to_string(),
                OApiType::Boolean(_) => "boolean".to_string(),
                OApiType::Object(_) => "object".to_string(),
                OApiType::Array(_) => "array".to_string(),
            },
            SchemaKind::AllOf { .. } => "object".to_string(),
            SchemaKind::OneOf { .. } => "oneOf".to_string(),
            SchemaKind::AnyOf { .. } => "anyOf".to_string(),
            SchemaKind::Not { .. } => "not".to_string(),
            SchemaKind::Any(_) => "any".to_string(),
        }
    }

    /// Resolve a parameter reference
    fn resolve_parameter_ref(&self, reference: &str) -> Option<ApiParameter> {
        // Extract parameter name from reference like "#/components/parameters/ParamName"
        let name = reference.rsplit('/').next()?;

        self.spec
            .components
            .as_ref()?
            .parameters
            .get(name)
            .and_then(|p| match p {
                ReferenceOr::Item(param) => self.parse_parameter(&ReferenceOr::Item(param.clone())),
                ReferenceOr::Reference { reference } => self.resolve_parameter_ref(reference),
            })
    }

    /// Parse request body
    fn parse_request_body(
        &self,
        body: &ReferenceOr<openapiv3::RequestBody>,
    ) -> Option<ApiRequestBody> {
        let body = match body {
            ReferenceOr::Item(b) => b,
            ReferenceOr::Reference { .. } => return None, // TODO: resolve reference
        };

        // Get the first content type (usually application/json)
        let (content_type, media) = body.content.iter().next()?;

        let schema_name = media.schema.as_ref().and_then(|s| match s {
            ReferenceOr::Reference { reference } => extract_ref_name(reference),
            ReferenceOr::Item(_) => None,
        });

        Some(ApiRequestBody {
            content_type: content_type.clone(),
            required: body.required,
            schema_name,
            description: body.description.clone(),
        })
    }

    /// Parse response
    fn parse_response(
        &self,
        status: &OApiStatusCode,
        response: &ReferenceOr<openapiv3::Response>,
    ) -> ApiResponse {
        let status_code = match status {
            OApiStatusCode::Code(code) => *code,
            OApiStatusCode::Range(_) => 200, // Default for ranges
        };

        let (description, content_type, schema_name) = match response {
            ReferenceOr::Item(r) => {
                let ct_and_schema = r.content.iter().next().map(|(ct, media)| {
                    let schema_name = media.schema.as_ref().and_then(|s| match s {
                        ReferenceOr::Reference { reference } => extract_ref_name(reference),
                        ReferenceOr::Item(_) => None,
                    });
                    (Some(ct.clone()), schema_name)
                });

                (
                    r.description.clone(),
                    ct_and_schema.as_ref().and_then(|(ct, _)| ct.clone()),
                    ct_and_schema.and_then(|(_, sn)| sn),
                )
            }
            ReferenceOr::Reference { reference } => {
                // Try to get description from reference name
                let name = extract_ref_name(reference).unwrap_or_default();
                (name, None, None)
            }
        };

        ApiResponse {
            status_code,
            description,
            content_type,
            schema_name,
        }
    }

    /// Parse all schemas
    fn parse_schemas(&self) -> Vec<ApiSchema> {
        let Some(components) = &self.spec.components else {
            return Vec::new();
        };

        components
            .schemas
            .iter()
            .map(|(name, schema)| self.parse_schema(name, schema))
            .collect()
    }

    /// Parse a single schema
    fn parse_schema(&self, name: &str, schema: &ReferenceOr<Schema>) -> ApiSchema {
        let schema = match schema {
            ReferenceOr::Item(s) => s,
            ReferenceOr::Reference { reference } => {
                // Return a reference schema
                return ApiSchema {
                    name: name.to_string(),
                    schema_type: SchemaType::Ref(
                        extract_ref_name(reference).unwrap_or_else(|| reference.clone()),
                    ),
                    properties: Vec::new(),
                    required: Vec::new(),
                    description: None,
                    items: None,
                    used_in: Vec::new(),
                    source: ApiSource::OpenApi {
                        url: self.source_url.clone(),
                        version: self.spec.openapi.clone(),
                    },
                    example: None,
                };
            }
        };

        let (schema_type, properties, items) = self.extract_schema_details(schema);
        let required = self.extract_required(schema);

        ApiSchema {
            name: name.to_string(),
            schema_type,
            properties,
            required,
            description: schema.schema_data.description.clone(),
            items,
            used_in: Vec::new(),
            source: ApiSource::OpenApi {
                url: self.source_url.clone(),
                version: self.spec.openapi.clone(),
            },
            example: schema.schema_data.example.clone(),
        }
    }

    /// Extract schema details (type, properties, items)
    fn extract_schema_details(
        &self,
        schema: &Schema,
    ) -> (SchemaType, Vec<SchemaProperty>, Option<Box<ApiSchema>>) {
        match &schema.schema_kind {
            SchemaKind::Type(t) => match t {
                OApiType::String(_) => (SchemaType::String, Vec::new(), None),
                OApiType::Number(_) => (SchemaType::Number, Vec::new(), None),
                OApiType::Integer(_) => (SchemaType::Integer, Vec::new(), None),
                OApiType::Boolean(_) => (SchemaType::Boolean, Vec::new(), None),
                OApiType::Object(obj) => {
                    let properties = self.extract_object_properties(obj, &schema.schema_data);
                    (SchemaType::Object, properties, None)
                }
                OApiType::Array(arr) => {
                    let items = arr.items.as_ref().and_then(|i| match i {
                        ReferenceOr::Item(s) => {
                            let (st, props, _) = self.extract_schema_details(s);
                            Some(Box::new(ApiSchema {
                                name: "items".to_string(),
                                schema_type: st,
                                properties: props,
                                required: Vec::new(),
                                description: None,
                                items: None,
                                used_in: Vec::new(),
                                source: ApiSource::Internal,
                                example: None,
                            }))
                        }
                        ReferenceOr::Reference { reference } => {
                            let ref_name = extract_ref_name(reference)?;
                            Some(Box::new(ApiSchema {
                                name: ref_name.clone(),
                                schema_type: SchemaType::Ref(ref_name),
                                properties: Vec::new(),
                                required: Vec::new(),
                                description: None,
                                items: None,
                                used_in: Vec::new(),
                                source: ApiSource::Internal,
                                example: None,
                            }))
                        }
                    });
                    (SchemaType::Array, Vec::new(), items)
                }
            },
            SchemaKind::AllOf { all_of } => {
                // Merge all schemas
                let mut properties = Vec::new();
                for schema_ref in all_of {
                    if let ReferenceOr::Item(s) = schema_ref {
                        let (_, props, _) = self.extract_schema_details(s);
                        properties.extend(props);
                    }
                }
                (SchemaType::Object, properties, None)
            }
            _ => (SchemaType::Object, Vec::new(), None),
        }
    }

    /// Extract properties from an object schema
    fn extract_object_properties(
        &self,
        obj: &openapiv3::ObjectType,
        _data: &openapiv3::SchemaData,
    ) -> Vec<SchemaProperty> {
        obj.properties
            .iter()
            .map(|(name, schema)| {
                let (schema_type, format, enum_values) = match schema {
                    ReferenceOr::Item(s) => {
                        let (st, _, _) = self.extract_schema_details(s);
                        let format = s.schema_data.description.clone();
                        (st, format, Vec::new())
                    }
                    ReferenceOr::Reference { reference } => {
                        let ref_name =
                            extract_ref_name(reference).unwrap_or_else(|| reference.clone());
                        (SchemaType::Ref(ref_name), None, Vec::new())
                    }
                };

                let required = obj.required.iter().any(|r| r == name);

                SchemaProperty {
                    name: name.clone(),
                    schema_type,
                    required,
                    description: None,
                    format,
                    enum_values,
                    default: None,
                    example: None,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                }
            })
            .collect()
    }

    /// Extract required fields
    fn extract_required(&self, schema: &Schema) -> Vec<String> {
        match &schema.schema_kind {
            SchemaKind::Type(OApiType::Object(obj)) => obj.required.clone(),
            _ => Vec::new(),
        }
    }
}

#[async_trait]
impl super::ApiAdapter for OpenApiAdapter {
    async fn parse(&self) -> Result<ApiRegistry> {
        let mut registry = ApiRegistry::new();

        // Parse endpoints
        let endpoints = self.parse_endpoints();
        registry.add_endpoints(endpoints);

        // Parse schemas
        let schemas = self.parse_schemas();
        registry.add_schemas(schemas);

        // Update schema usage
        registry.update_schema_usage();

        Ok(registry)
    }

    fn description(&self) -> &str {
        "OpenAPI 3.x Adapter"
    }
}

/// Extract schema name from a reference string
fn extract_ref_name(reference: &str) -> Option<String> {
    reference.rsplit('/').next().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ApiAdapter;

    const SAMPLE_SPEC: &str = r#"{
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/users": {
                "get": {
                    "operationId": "listUsers",
                    "summary": "List all users",
                    "tags": ["users"],
                    "responses": {
                        "200": {
                            "description": "Success"
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "name": {"type": "string"}
                    },
                    "required": ["id"]
                }
            }
        }
    }"#;

    #[test]
    fn test_parse_from_json() {
        let adapter = OpenApiAdapter::from_json(SAMPLE_SPEC).unwrap();
        assert_eq!(adapter.version(), "3.0.0");
        assert_eq!(adapter.info().title, "Test API");
    }

    #[tokio::test]
    async fn test_parse_endpoints() {
        let adapter = OpenApiAdapter::from_json(SAMPLE_SPEC).unwrap();
        let registry = adapter.parse().await.unwrap();

        assert_eq!(registry.endpoints().len(), 1);
        let endpoint = &registry.endpoints()[0];
        assert_eq!(endpoint.path, "/users");
        assert_eq!(endpoint.method, HttpMethod::Get);
        assert_eq!(endpoint.operation_id, Some("listUsers".to_string()));
    }

    #[tokio::test]
    async fn test_parse_schemas() {
        let adapter = OpenApiAdapter::from_json(SAMPLE_SPEC).unwrap();
        let registry = adapter.parse().await.unwrap();

        assert_eq!(registry.schemas().len(), 1);
        let schema = registry.get_schema("User").unwrap();
        assert_eq!(schema.name, "User");
        assert_eq!(schema.required, vec!["id".to_string()]);
    }
}

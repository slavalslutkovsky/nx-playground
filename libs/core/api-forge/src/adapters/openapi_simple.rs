//! Simple OpenAPI Parser using serde_json
//!
//! A flexible parser that works with any OpenAPI 3.x spec without strict typing.

use crate::error::{ApiForgeError, Result};
use crate::registry::ApiRegistry;
use crate::schema::{
    ApiEndpoint, ApiParameter, ApiRequestBody, ApiResponse, ApiSchema, ApiSource, HttpMethod,
    ParameterLocation, SchemaProperty, SchemaType,
};
use serde_json::Value;

/// Simple adapter that parses OpenAPI specs as raw JSON
pub struct SimpleOpenApiAdapter {
    spec: Value,
    source_url: Option<String>,
}

impl SimpleOpenApiAdapter {
    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let spec: Value =
            serde_json::from_str(json).map_err(|e| ApiForgeError::OpenApiParse(e.to_string()))?;
        Ok(Self {
            spec,
            source_url: None,
        })
    }

    /// Create from JSON value
    pub fn from_value(spec: Value) -> Self {
        Self {
            spec,
            source_url: None,
        }
    }

    /// Set source URL
    pub fn with_source(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Get version string
    pub fn version(&self) -> &str {
        self.spec
            .get("openapi")
            .and_then(|v| v.as_str())
            .unwrap_or("3.0.0")
    }

    /// Parse into registry
    pub fn parse(&self) -> Result<ApiRegistry> {
        let mut registry = ApiRegistry::new();

        // Parse paths/endpoints
        if let Some(paths) = self.spec.get("paths").and_then(|p| p.as_object()) {
            for (path, path_item) in paths {
                if let Some(item) = path_item.as_object() {
                    self.parse_path_item(&mut registry, path, item);
                }
            }
        }

        // Parse schemas from components
        if let Some(components) = self.spec.get("components").and_then(|c| c.as_object())
            && let Some(schemas) = components.get("schemas").and_then(|s| s.as_object())
        {
            for (name, schema) in schemas {
                if let Some(api_schema) = self.parse_schema(name, schema) {
                    registry.add_schema(api_schema);
                }
            }
        }

        registry.update_schema_usage();
        Ok(registry)
    }

    fn parse_path_item(
        &self,
        registry: &mut ApiRegistry,
        path: &str,
        item: &serde_json::Map<String, Value>,
    ) {
        let methods = [
            ("get", HttpMethod::Get),
            ("post", HttpMethod::Post),
            ("put", HttpMethod::Put),
            ("patch", HttpMethod::Patch),
            ("delete", HttpMethod::Delete),
            ("head", HttpMethod::Head),
            ("options", HttpMethod::Options),
        ];

        // Get path-level parameters
        let path_params: Vec<ApiParameter> = item
            .get("parameters")
            .and_then(|p| p.as_array())
            .map(|arr| arr.iter().filter_map(|p| self.parse_parameter(p)).collect())
            .unwrap_or_default();

        for (method_name, method) in methods {
            if let Some(operation) = item.get(method_name).and_then(|o| o.as_object()) {
                let endpoint = self.parse_operation(path, method, operation, &path_params);
                registry.add_endpoint(endpoint);
            }
        }
    }

    fn parse_operation(
        &self,
        path: &str,
        method: HttpMethod,
        op: &serde_json::Map<String, Value>,
        path_params: &[ApiParameter],
    ) -> ApiEndpoint {
        let operation_id = op
            .get("operationId")
            .and_then(|v| v.as_str())
            .map(String::from);
        let summary = op.get("summary").and_then(|v| v.as_str()).map(String::from);
        let description = op
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);
        let deprecated = op
            .get("deprecated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let tags: Vec<String> = op
            .get("tags")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Combine path and operation parameters
        let mut parameters = path_params.to_vec();
        if let Some(params) = op.get("parameters").and_then(|p| p.as_array()) {
            parameters.extend(params.iter().filter_map(|p| self.parse_parameter(p)));
        }

        let request_body = op
            .get("requestBody")
            .and_then(|rb| self.parse_request_body(rb));

        let responses: Vec<ApiResponse> = op
            .get("responses")
            .and_then(|r| r.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(status, resp)| self.parse_response(status, resp))
                    .collect()
            })
            .unwrap_or_default();

        ApiEndpoint {
            path: path.to_string(),
            method,
            operation_id,
            summary,
            description,
            tags,
            parameters,
            request_body,
            responses,
            security: vec![],
            source: ApiSource::OpenApi {
                url: self.source_url.clone(),
                version: self.version().to_string(),
            },
            deprecated,
        }
    }

    fn parse_parameter(&self, param: &Value) -> Option<ApiParameter> {
        let obj = param.as_object()?;

        // Handle $ref
        if let Some(ref_path) = obj.get("$ref").and_then(|r| r.as_str()) {
            return self.resolve_parameter_ref(ref_path);
        }

        let name = obj.get("name").and_then(|n| n.as_str())?.to_string();
        let in_value = obj.get("in").and_then(|i| i.as_str())?;
        let location = match in_value {
            "path" => ParameterLocation::Path,
            "query" => ParameterLocation::Query,
            "header" => ParameterLocation::Header,
            "cookie" => ParameterLocation::Cookie,
            _ => return None,
        };

        let required = obj
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let description = obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(String::from);

        let schema_type = obj
            .get("schema")
            .and_then(|s| s.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("string")
            .to_string();

        Some(ApiParameter {
            name,
            location,
            required,
            description,
            schema_type,
            default: obj.get("default").cloned(),
            example: obj.get("example").cloned(),
        })
    }

    fn resolve_parameter_ref(&self, ref_path: &str) -> Option<ApiParameter> {
        // Parse refs like "#/components/parameters/ParamName"
        let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
        if parts.len() < 3 {
            return None;
        }

        let mut current = &self.spec;
        for part in &parts {
            current = current.get(*part)?;
        }

        self.parse_parameter(current)
    }

    fn parse_request_body(&self, body: &Value) -> Option<ApiRequestBody> {
        let obj = body.as_object()?;

        // Get first content type
        let content = obj.get("content").and_then(|c| c.as_object())?;
        let (content_type, media) = content.iter().next()?;

        let schema_name = media
            .get("schema")
            .and_then(|s| s.get("$ref"))
            .and_then(|r| r.as_str())
            .and_then(|r| r.rsplit('/').next())
            .map(String::from);

        let required = obj
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let description = obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(String::from);

        Some(ApiRequestBody {
            content_type: content_type.clone(),
            required,
            schema_name,
            description,
        })
    }

    fn parse_response(&self, status: &str, resp: &Value) -> ApiResponse {
        let status_code: u16 = status.parse().unwrap_or(200);

        let (description, content_type, schema_name) = if let Some(obj) = resp.as_object() {
            let desc = obj
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();

            let (ct, sn) = obj
                .get("content")
                .and_then(|c| c.as_object())
                .and_then(|content| content.iter().next())
                .map(|(ct, media)| {
                    let sn = media
                        .get("schema")
                        .and_then(|s| s.get("$ref"))
                        .and_then(|r| r.as_str())
                        .and_then(|r| r.rsplit('/').next())
                        .map(String::from);
                    (Some(ct.clone()), sn)
                })
                .unwrap_or((None, None));

            (desc, ct, sn)
        } else {
            (String::new(), None, None)
        };

        ApiResponse {
            status_code,
            description,
            content_type,
            schema_name,
        }
    }

    fn parse_schema(&self, name: &str, schema: &Value) -> Option<ApiSchema> {
        let obj = schema.as_object()?;

        let schema_type = obj
            .get("type")
            .and_then(|t| t.as_str())
            .map(|t| match t {
                "object" => SchemaType::Object,
                "array" => SchemaType::Array,
                "string" => SchemaType::String,
                "integer" => SchemaType::Integer,
                "number" => SchemaType::Number,
                "boolean" => SchemaType::Boolean,
                _ => SchemaType::Object,
            })
            .unwrap_or(SchemaType::Object);

        let description = obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(String::from);

        let required: Vec<String> = obj
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let properties: Vec<SchemaProperty> = obj
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|props| {
                props
                    .iter()
                    .map(|(prop_name, prop_schema)| {
                        self.parse_schema_property(prop_name, prop_schema, &required)
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(ApiSchema {
            name: name.to_string(),
            schema_type,
            properties,
            required,
            description,
            items: None,
            used_in: vec![],
            source: ApiSource::OpenApi {
                url: self.source_url.clone(),
                version: self.version().to_string(),
            },
            example: obj.get("example").cloned(),
        })
    }

    fn parse_schema_property(
        &self,
        name: &str,
        schema: &Value,
        required_fields: &[String],
    ) -> SchemaProperty {
        let obj = schema.as_object();

        let schema_type = obj
            .and_then(|o| o.get("type"))
            .and_then(|t| t.as_str())
            .map(|t| match t {
                "string" => SchemaType::String,
                "integer" => SchemaType::Integer,
                "number" => SchemaType::Number,
                "boolean" => SchemaType::Boolean,
                "array" => SchemaType::Array,
                "object" => SchemaType::Object,
                _ => SchemaType::String,
            })
            .or_else(|| {
                obj.and_then(|o| o.get("$ref"))
                    .and_then(|r| r.as_str())
                    .and_then(|r| r.rsplit('/').next())
                    .map(|n| SchemaType::Ref(n.to_string()))
            })
            .unwrap_or(SchemaType::String);

        let description = obj
            .and_then(|o| o.get("description"))
            .and_then(|d| d.as_str())
            .map(String::from);

        let format = obj
            .and_then(|o| o.get("format"))
            .and_then(|f| f.as_str())
            .map(String::from);

        SchemaProperty {
            name: name.to_string(),
            schema_type,
            required: required_fields.contains(&name.to_string()),
            description,
            format,
            enum_values: vec![],
            default: obj.and_then(|o| o.get("default").cloned()),
            example: obj.and_then(|o| o.get("example").cloned()),
            minimum: obj.and_then(|o| o.get("minimum")).and_then(|m| m.as_f64()),
            maximum: obj.and_then(|o| o.get("maximum")).and_then(|m| m.as_f64()),
            min_length: obj
                .and_then(|o| o.get("minLength"))
                .and_then(|m| m.as_u64())
                .map(|v| v as usize),
            max_length: obj
                .and_then(|o| o.get("maxLength"))
                .and_then(|m| m.as_u64())
                .map(|v| v as usize),
            pattern: obj
                .and_then(|o| o.get("pattern"))
                .and_then(|p| p.as_str())
                .map(String::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SPEC: &str = r#"{
        "openapi": "3.0.0",
        "info": {"title": "Test API", "version": "1.0.0"},
        "paths": {
            "/users": {
                "get": {
                    "operationId": "listUsers",
                    "summary": "List all users",
                    "tags": ["users"],
                    "responses": {
                        "200": {"description": "Success"}
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
    fn test_parse_simple_spec() {
        let adapter = SimpleOpenApiAdapter::from_json(SAMPLE_SPEC).unwrap();
        let registry = adapter.parse().unwrap();

        assert_eq!(registry.endpoints().len(), 1);
        assert_eq!(registry.schemas().len(), 1);

        let endpoint = &registry.endpoints()[0];
        assert_eq!(endpoint.path, "/users");
        assert_eq!(endpoint.method, HttpMethod::Get);
    }
}

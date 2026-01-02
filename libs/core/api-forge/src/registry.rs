//! API Registry - Central storage for API metadata
//!
//! The registry aggregates endpoints and schemas from multiple sources
//! and provides methods for querying and searching the API.

use crate::schema::{
    ApiEndpoint, ApiSchema, ApiStats, EndpointListResponse, HttpMethod, SchemaListResponse,
    SearchResponse, SearchResult,
};
use std::collections::HashMap;

/// Central registry for API endpoints and schemas
#[derive(Debug, Clone, Default)]
pub struct ApiRegistry {
    /// All registered endpoints
    endpoints: Vec<ApiEndpoint>,
    /// All registered schemas by name
    schemas: HashMap<String, ApiSchema>,
    /// Index: tag -> endpoint indices
    tag_index: HashMap<String, Vec<usize>>,
    /// Index: method -> endpoint indices
    method_index: HashMap<HttpMethod, Vec<usize>>,
}

impl ApiRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an endpoint to the registry
    pub fn add_endpoint(&mut self, endpoint: ApiEndpoint) {
        let idx = self.endpoints.len();

        // Update tag index
        for tag in &endpoint.tags {
            self.tag_index.entry(tag.clone()).or_default().push(idx);
        }

        // Update method index
        self.method_index
            .entry(endpoint.method)
            .or_default()
            .push(idx);

        self.endpoints.push(endpoint);
    }

    /// Add multiple endpoints to the registry
    pub fn add_endpoints(&mut self, endpoints: impl IntoIterator<Item = ApiEndpoint>) {
        for endpoint in endpoints {
            self.add_endpoint(endpoint);
        }
    }

    /// Add a schema to the registry
    pub fn add_schema(&mut self, schema: ApiSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Add multiple schemas to the registry
    pub fn add_schemas(&mut self, schemas: impl IntoIterator<Item = ApiSchema>) {
        for schema in schemas {
            self.add_schema(schema);
        }
    }

    /// Get all endpoints
    pub fn endpoints(&self) -> &[ApiEndpoint] {
        &self.endpoints
    }

    /// Get all schemas
    pub fn schemas(&self) -> &HashMap<String, ApiSchema> {
        &self.schemas
    }

    /// Get endpoint by path and method
    pub fn get_endpoint(&self, path: &str, method: HttpMethod) -> Option<&ApiEndpoint> {
        self.endpoints
            .iter()
            .find(|e| e.path == path && e.method == method)
    }

    /// Get endpoint by operation ID
    pub fn get_endpoint_by_operation_id(&self, operation_id: &str) -> Option<&ApiEndpoint> {
        self.endpoints
            .iter()
            .find(|e| e.operation_id.as_deref() == Some(operation_id))
    }

    /// Get schema by name
    pub fn get_schema(&self, name: &str) -> Option<&ApiSchema> {
        self.schemas.get(name)
    }

    /// Get endpoints by tag
    pub fn get_endpoints_by_tag(&self, tag: &str) -> Vec<&ApiEndpoint> {
        self.tag_index
            .get(tag)
            .map(|indices| indices.iter().map(|&i| &self.endpoints[i]).collect())
            .unwrap_or_default()
    }

    /// Get endpoints by HTTP method
    pub fn get_endpoints_by_method(&self, method: HttpMethod) -> Vec<&ApiEndpoint> {
        self.method_index
            .get(&method)
            .map(|indices| indices.iter().map(|&i| &self.endpoints[i]).collect())
            .unwrap_or_default()
    }

    /// Get all unique tags
    pub fn tags(&self) -> Vec<&str> {
        self.tag_index.keys().map(|s| s.as_str()).collect()
    }

    /// List endpoints with pagination
    pub fn list_endpoints(
        &self,
        tag: Option<&str>,
        method: Option<HttpMethod>,
        offset: usize,
        limit: usize,
    ) -> EndpointListResponse {
        let filtered: Vec<&ApiEndpoint> = self
            .endpoints
            .iter()
            .filter(|e| {
                let tag_match = tag.map(|t| e.tags.contains(&t.to_string())).unwrap_or(true);
                let method_match = method.map(|m| e.method == m).unwrap_or(true);
                tag_match && method_match
            })
            .collect();

        let total = filtered.len();
        let endpoints: Vec<ApiEndpoint> = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        EndpointListResponse { total, endpoints }
    }

    /// List schemas with pagination
    pub fn list_schemas(&self, offset: usize, limit: usize) -> SchemaListResponse {
        let total = self.schemas.len();
        let schemas: Vec<ApiSchema> = self
            .schemas
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        SchemaListResponse { total, schemas }
    }

    /// Search endpoints and schemas
    pub fn search(&self, query: &str, limit: usize) -> SearchResponse {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search endpoints
        for endpoint in &self.endpoints {
            let mut matched = false;
            let mut match_field = String::new();

            // Search in path
            if endpoint.path.to_lowercase().contains(&query_lower) {
                matched = true;
                match_field = "path".to_string();
            }
            // Search in summary
            else if endpoint
                .summary
                .as_ref()
                .map(|s| s.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            {
                matched = true;
                match_field = "summary".to_string();
            }
            // Search in description
            else if endpoint
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            {
                matched = true;
                match_field = "description".to_string();
            }
            // Search in tags
            else if endpoint
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
            {
                matched = true;
                match_field = "tags".to_string();
            }
            // Search in operation_id
            else if endpoint
                .operation_id
                .as_ref()
                .map(|op| op.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            {
                matched = true;
                match_field = "operation_id".to_string();
            }

            if matched {
                results.push(SearchResult::Endpoint {
                    path: endpoint.path.clone(),
                    method: endpoint.method.to_string(),
                    summary: endpoint.summary.clone(),
                    tags: endpoint.tags.clone(),
                    match_field,
                });
            }

            if results.len() >= limit {
                break;
            }
        }

        // Search schemas if we have room
        if results.len() < limit {
            for schema in self.schemas.values() {
                let mut matched = false;
                let mut match_field = String::new();

                // Search in name
                if schema.name.to_lowercase().contains(&query_lower) {
                    matched = true;
                    match_field = "name".to_string();
                }
                // Search in description
                else if schema
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                {
                    matched = true;
                    match_field = "description".to_string();
                }
                // Search in property names
                else if schema
                    .properties
                    .iter()
                    .any(|p| p.name.to_lowercase().contains(&query_lower))
                {
                    matched = true;
                    match_field = "properties".to_string();
                }

                if matched {
                    results.push(SearchResult::Schema {
                        name: schema.name.clone(),
                        description: schema.description.clone(),
                        match_field,
                    });
                }

                if results.len() >= limit {
                    break;
                }
            }
        }

        SearchResponse {
            query: query.to_string(),
            total: results.len(),
            results,
        }
    }

    /// Get API statistics
    pub fn stats(&self) -> ApiStats {
        let mut endpoints_by_method: HashMap<String, usize> = HashMap::new();
        let mut endpoints_by_tag: HashMap<String, usize> = HashMap::new();
        let mut deprecated_count = 0;
        let mut documented_count = 0;

        for endpoint in &self.endpoints {
            // Count by method
            *endpoints_by_method
                .entry(endpoint.method.to_string())
                .or_default() += 1;

            // Count by tag
            for tag in &endpoint.tags {
                *endpoints_by_tag.entry(tag.clone()).or_default() += 1;
            }

            // Count deprecated
            if endpoint.deprecated {
                deprecated_count += 1;
            }

            // Count documented (has summary or description)
            if endpoint.summary.is_some() || endpoint.description.is_some() {
                documented_count += 1;
            }
        }

        let documentation_coverage = if self.endpoints.is_empty() {
            100.0
        } else {
            (documented_count as f64 / self.endpoints.len() as f64) * 100.0
        };

        ApiStats {
            total_endpoints: self.endpoints.len(),
            total_schemas: self.schemas.len(),
            endpoints_by_method,
            endpoints_by_tag,
            deprecated_endpoints: deprecated_count,
            documentation_coverage,
        }
    }

    /// Merge another registry into this one
    pub fn merge(&mut self, other: ApiRegistry) {
        self.add_endpoints(other.endpoints);
        self.add_schemas(other.schemas.into_values());
    }

    /// Update schema usage information based on endpoints
    pub fn update_schema_usage(&mut self) {
        // Collect schema usage from endpoints
        let mut usage: HashMap<String, Vec<String>> = HashMap::new();

        for endpoint in &self.endpoints {
            let endpoint_key = endpoint.key();

            // Check request body
            if let Some(ref body) = endpoint.request_body
                && let Some(ref schema_name) = body.schema_name
            {
                usage
                    .entry(schema_name.clone())
                    .or_default()
                    .push(endpoint_key.clone());
            }

            // Check responses
            for response in &endpoint.responses {
                if let Some(ref schema_name) = response.schema_name {
                    usage
                        .entry(schema_name.clone())
                        .or_default()
                        .push(endpoint_key.clone());
                }
            }
        }

        // Update schemas with usage info
        for (schema_name, endpoints) in usage {
            if let Some(schema) = self.schemas.get_mut(&schema_name) {
                schema.used_in = endpoints;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_endpoint(path: &str, method: HttpMethod) -> ApiEndpoint {
        ApiEndpoint {
            path: path.to_string(),
            method,
            operation_id: Some(format!(
                "{}_{}",
                method.to_string().to_lowercase(),
                path.replace('/', "_")
            )),
            summary: Some(format!("{} {}", method, path)),
            description: None,
            tags: vec!["test".to_string()],
            parameters: vec![],
            request_body: None,
            responses: vec![],
            security: vec![],
            source: crate::schema::ApiSource::Internal,
            deprecated: false,
        }
    }

    #[test]
    fn test_add_and_get_endpoint() {
        let mut registry = ApiRegistry::new();
        let endpoint = create_test_endpoint("/api/users", HttpMethod::Get);
        registry.add_endpoint(endpoint);

        let result = registry.get_endpoint("/api/users", HttpMethod::Get);
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/api/users");
    }

    #[test]
    fn test_search_endpoints() {
        let mut registry = ApiRegistry::new();
        registry.add_endpoint(create_test_endpoint("/api/users", HttpMethod::Get));
        registry.add_endpoint(create_test_endpoint("/api/projects", HttpMethod::Get));
        registry.add_endpoint(create_test_endpoint("/api/users", HttpMethod::Post));

        let results = registry.search("users", 10);
        assert_eq!(results.total, 2);
    }

    #[test]
    fn test_stats() {
        let mut registry = ApiRegistry::new();
        registry.add_endpoint(create_test_endpoint("/api/users", HttpMethod::Get));
        registry.add_endpoint(create_test_endpoint("/api/users", HttpMethod::Post));
        registry.add_endpoint(create_test_endpoint("/api/projects", HttpMethod::Get));

        let stats = registry.stats();
        assert_eq!(stats.total_endpoints, 3);
        assert_eq!(stats.endpoints_by_method.get("GET"), Some(&2));
        assert_eq!(stats.endpoints_by_method.get("POST"), Some(&1));
    }
}

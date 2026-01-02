//! Help endpoint handlers
//!
//! Provides HTTP handlers for the `/help` endpoints that expose
//! API documentation and metadata.

use crate::registry::ApiRegistry;
use crate::schema::{
    ApiEndpoint, ApiSchema, ApiStats, EndpointListResponse, HttpMethod, SchemaListResponse,
    SearchResponse,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::{IntoParams, OpenApi, ToSchema};

/// State for the help handlers
#[derive(Clone)]
pub struct HelpState {
    registry: Arc<RwLock<ApiRegistry>>,
    api_name: String,
    api_version: String,
}

impl HelpState {
    /// Create a new HelpState with an empty registry
    pub fn new(api_name: impl Into<String>, api_version: impl Into<String>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(ApiRegistry::new())),
            api_name: api_name.into(),
            api_version: api_version.into(),
        }
    }

    /// Create a new HelpState with an existing registry
    pub fn with_registry(
        registry: ApiRegistry,
        api_name: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
            api_name: api_name.into(),
            api_version: api_version.into(),
        }
    }

    /// Get the registry
    pub fn registry(&self) -> &Arc<RwLock<ApiRegistry>> {
        &self.registry
    }

    /// Update the registry
    pub async fn set_registry(&self, registry: ApiRegistry) {
        let mut lock = self.registry.write().await;
        *lock = registry;
    }

    /// Initialize from an OpenAPI spec JSON using the simple adapter
    pub async fn init_from_openapi_json(&self, json: &str) -> crate::Result<()> {
        let adapter = crate::adapters::SimpleOpenApiAdapter::from_json(json)?;
        let registry = adapter.parse()?;
        self.set_registry(registry).await;
        Ok(())
    }
}

/// Query parameters for listing endpoints
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListEndpointsQuery {
    /// Filter by tag
    pub tag: Option<String>,
    /// Filter by HTTP method
    pub method: Option<String>,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
    /// Limit for pagination (default 50)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// Query parameters for listing schemas
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListSchemasQuery {
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
    /// Limit for pagination (default 50)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Query parameters for search
#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Limit for results (default 20)
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    20
}

/// Response for the help index
#[derive(Debug, Serialize, ToSchema)]
pub struct HelpIndexResponse {
    pub name: String,
    pub version: String,
    pub stats: ApiStats,
    pub links: HelpLinks,
}

/// Links to help resources
#[derive(Debug, Serialize, ToSchema)]
pub struct HelpLinks {
    pub endpoints: String,
    pub schemas: String,
    pub search: String,
    pub openapi: String,
    pub swagger_ui: String,
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
pub struct HelpErrorResponse {
    pub error: String,
    pub message: String,
}

impl HelpErrorResponse {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            error: "not_found".to_string(),
            message: message.into(),
        }
    }
}

/// Create the help router
pub fn help_router(state: HelpState) -> Router {
    Router::new()
        .route("/", get(help_index).head(help_index))
        .route("/endpoints", get(list_endpoints))
        .route("/endpoints/{*path}", get(get_endpoint))
        .route("/schemas", get(list_schemas))
        .route("/schemas/{name}", get(get_schema))
        .route("/search", get(search))
        .route("/stats", get(get_stats))
        .route("/html", get(help_html))
        .with_state(state)
}

/// GET / - Help index with stats and links
#[utoipa::path(
    get,
    path = "/",
    tag = "Help",
    responses(
        (status = 200, description = "Help index", body = HelpIndexResponse),
    )
)]
async fn help_index(State(state): State<HelpState>) -> Json<HelpIndexResponse> {
    let registry = state.registry.read().await;
    let stats = registry.stats();

    Json(HelpIndexResponse {
        name: state.api_name.clone(),
        version: state.api_version.clone(),
        stats,
        links: HelpLinks {
            endpoints: "/help/endpoints".to_string(),
            schemas: "/help/schemas".to_string(),
            search: "/help/search?q=".to_string(),
            openapi: "/api-docs/openapi.json".to_string(),
            swagger_ui: "/swagger-ui".to_string(),
        },
    })
}

/// GET /endpoints - List all endpoints
#[utoipa::path(
    get,
    path = "/endpoints",
    tag = "Help",
    params(ListEndpointsQuery),
    responses(
        (status = 200, description = "List of endpoints", body = EndpointListResponse),
    )
)]
async fn list_endpoints(
    State(state): State<HelpState>,
    Query(query): Query<ListEndpointsQuery>,
) -> Json<EndpointListResponse> {
    let registry = state.registry.read().await;

    // Parse method if provided
    let method = query
        .method
        .as_ref()
        .and_then(|m| m.to_uppercase().parse::<HttpMethod>().ok());

    let response = registry.list_endpoints(query.tag.as_deref(), method, query.offset, query.limit);
    Json(response)
}

/// GET /endpoints/{path} - Get a specific endpoint
#[utoipa::path(
    get,
    path = "/endpoints/{path}",
    tag = "Help",
    params(
        ("path" = String, Path, description = "Endpoint path (URL encoded)")
    ),
    responses(
        (status = 200, description = "Endpoint details", body = ApiEndpoint),
        (status = 404, description = "Endpoint not found", body = HelpErrorResponse),
    )
)]
async fn get_endpoint(State(state): State<HelpState>, Path(path): Path<String>) -> Response {
    let registry = state.registry.read().await;

    // The path comes without leading slash, add it back
    let full_path = format!("/{}", path);

    // Try to find endpoint for any method
    for method in [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Patch,
        HttpMethod::Delete,
        HttpMethod::Head,
        HttpMethod::Options,
    ] {
        if let Some(endpoint) = registry.get_endpoint(&full_path, method) {
            return Json(endpoint.clone()).into_response();
        }
    }

    // Return all endpoints matching this path
    let matching: Vec<_> = registry
        .endpoints()
        .iter()
        .filter(|e| e.path == full_path)
        .cloned()
        .collect();

    if matching.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(HelpErrorResponse::not_found(format!(
                "No endpoint found at path: {}",
                full_path
            ))),
        )
            .into_response();
    }

    Json(matching).into_response()
}

/// GET /schemas - List all schemas
#[utoipa::path(
    get,
    path = "/schemas",
    tag = "Help",
    params(ListSchemasQuery),
    responses(
        (status = 200, description = "List of schemas", body = SchemaListResponse),
    )
)]
async fn list_schemas(
    State(state): State<HelpState>,
    Query(query): Query<ListSchemasQuery>,
) -> Json<SchemaListResponse> {
    let registry = state.registry.read().await;
    let response = registry.list_schemas(query.offset, query.limit);
    Json(response)
}

/// GET /schemas/{name} - Get a specific schema
#[utoipa::path(
    get,
    path = "/schemas/{name}",
    tag = "Help",
    params(
        ("name" = String, Path, description = "Schema name")
    ),
    responses(
        (status = 200, description = "Schema details", body = ApiSchema),
        (status = 404, description = "Schema not found", body = HelpErrorResponse),
    )
)]
async fn get_schema(State(state): State<HelpState>, Path(name): Path<String>) -> Response {
    let registry = state.registry.read().await;

    match registry.get_schema(&name) {
        Some(schema) => Json(schema.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(HelpErrorResponse::not_found(format!(
                "Schema not found: {}",
                name
            ))),
        )
            .into_response(),
    }
}

/// GET /search - Search endpoints and schemas
#[utoipa::path(
    get,
    path = "/search",
    tag = "Help",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
    )
)]
async fn search(
    State(state): State<HelpState>,
    Query(query): Query<SearchQuery>,
) -> Json<SearchResponse> {
    let registry = state.registry.read().await;
    let response = registry.search(&query.q, query.limit);
    Json(response)
}

/// GET /stats - Get API statistics
#[utoipa::path(
    get,
    path = "/stats",
    tag = "Help",
    responses(
        (status = 200, description = "API statistics", body = ApiStats),
    )
)]
async fn get_stats(State(state): State<HelpState>) -> Json<ApiStats> {
    let registry = state.registry.read().await;
    Json(registry.stats())
}

/// GET /html - Interactive HTML help page
#[utoipa::path(
    get,
    path = "/html",
    tag = "Help",
    responses(
        (status = 200, description = "Interactive HTML help page", content_type = "text/html"),
    )
)]
async fn help_html(State(state): State<HelpState>) -> Html<String> {
    let registry = state.registry.read().await;
    let stats = registry.stats();

    let endpoints_html: String = registry
        .endpoints()
        .iter()
        .map(|e| {
            let method_class = e.method.to_string().to_lowercase();
            let deprecated = if e.deprecated {
                r#" <span class="deprecated">deprecated</span>"#
            } else {
                ""
            };
            format!(
                r#"<tr>
                    <td><span class="method {}">{}</span></td>
                    <td><code>{}</code>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                method_class,
                e.method,
                e.path,
                deprecated,
                e.summary.as_deref().unwrap_or("-"),
                e.tags.join(", ")
            )
        })
        .collect();

    let schemas_html: String = registry
        .schemas()
        .values()
        .map(|s| {
            let props: Vec<_> = s.properties.iter().map(|p| p.name.as_str()).collect();
            format!(
                r#"<tr>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                s.name,
                s.description.as_deref().unwrap_or("-"),
                props.join(", ")
            )
        })
        .collect();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} API Help</title>
    <style>
        :root {{
            --bg: #0d1117;
            --bg-secondary: #161b22;
            --text: #c9d1d9;
            --text-muted: #8b949e;
            --border: #30363d;
            --accent: #58a6ff;
            --get: #3fb950;
            --post: #a371f7;
            --put: #f0883e;
            --patch: #db61a2;
            --delete: #f85149;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding: 2rem;
        }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{ color: var(--accent); margin-bottom: 0.5rem; }}
        h2 {{ color: var(--text); margin: 2rem 0 1rem; border-bottom: 1px solid var(--border); padding-bottom: 0.5rem; }}
        .version {{ color: var(--text-muted); font-size: 0.9rem; }}
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 1rem;
            margin: 1.5rem 0;
        }}
        .stat {{
            background: var(--bg-secondary);
            padding: 1rem;
            border-radius: 8px;
            border: 1px solid var(--border);
        }}
        .stat-value {{ font-size: 2rem; font-weight: bold; color: var(--accent); }}
        .stat-label {{ color: var(--text-muted); font-size: 0.85rem; }}
        .search {{
            margin: 1.5rem 0;
            display: flex;
            gap: 0.5rem;
        }}
        input[type="text"] {{
            flex: 1;
            padding: 0.75rem 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 6px;
            color: var(--text);
            font-size: 1rem;
        }}
        input[type="text"]:focus {{
            outline: none;
            border-color: var(--accent);
        }}
        button {{
            padding: 0.75rem 1.5rem;
            background: var(--accent);
            border: none;
            border-radius: 6px;
            color: var(--bg);
            font-weight: 600;
            cursor: pointer;
        }}
        button:hover {{ opacity: 0.9; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid var(--border);
        }}
        th {{ color: var(--text-muted); font-weight: 600; }}
        tr:hover {{ background: var(--bg-secondary); }}
        .method {{
            display: inline-block;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }}
        .method.get {{ background: var(--get); color: var(--bg); }}
        .method.post {{ background: var(--post); color: var(--bg); }}
        .method.put {{ background: var(--put); color: var(--bg); }}
        .method.patch {{ background: var(--patch); color: var(--bg); }}
        .method.delete {{ background: var(--delete); color: var(--bg); }}
        code {{
            background: var(--bg-secondary);
            padding: 0.2rem 0.4rem;
            border-radius: 4px;
            font-family: 'Fira Code', 'Monaco', monospace;
            font-size: 0.9rem;
        }}
        .deprecated {{
            background: var(--delete);
            color: var(--bg);
            padding: 0.1rem 0.3rem;
            border-radius: 3px;
            font-size: 0.7rem;
            margin-left: 0.5rem;
        }}
        .links {{
            display: flex;
            gap: 1rem;
            margin: 1rem 0;
        }}
        .links a {{
            color: var(--accent);
            text-decoration: none;
        }}
        .links a:hover {{ text-decoration: underline; }}
        #search-results {{
            margin-top: 1rem;
            display: none;
        }}
        #search-results.active {{ display: block; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{} API</h1>
        <p class="version">Version {}</p>

        <div class="stats">
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Endpoints</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Schemas</div>
            </div>
            <div class="stat">
                <div class="stat-value">{:.0}%</div>
                <div class="stat-label">Documented</div>
            </div>
        </div>

        <div class="links">
            <a href="/swagger-ui">Swagger UI</a>
            <a href="/redoc">ReDoc</a>
            <a href="/rapidoc">RapiDoc</a>
            <a href="/scalar">Scalar</a>
            <a href="/api-docs/openapi.json">OpenAPI JSON</a>
        </div>

        <div class="search">
            <input type="text" id="search-input" placeholder="Search endpoints and schemas...">
            <button onclick="doSearch()">Search</button>
        </div>
        <div id="search-results"></div>

        <h2>Endpoints</h2>
        <table>
            <thead>
                <tr>
                    <th>Method</th>
                    <th>Path</th>
                    <th>Summary</th>
                    <th>Tags</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <h2>Schemas</h2>
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Description</th>
                    <th>Properties</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>
    <script>
        function doSearch() {{
            const query = document.getElementById('search-input').value;
            if (!query) return;

            fetch(`/help/search?q=${{encodeURIComponent(query)}}`)
                .then(r => r.json())
                .then(data => {{
                    const results = document.getElementById('search-results');
                    if (data.results.length === 0) {{
                        results.innerHTML = '<p>No results found.</p>';
                    }} else {{
                        results.innerHTML = '<h3>Search Results (' + data.total + ')</h3><ul>' +
                            data.results.map(r => {{
                                if (r.type === 'endpoint') {{
                                    return `<li><span class="method ${{r.method.toLowerCase()}}">${{r.method}}</span> <code>${{r.path}}</code> - ${{r.summary || 'No summary'}}</li>`;
                                }} else {{
                                    return `<li><code>${{r.name}}</code> - ${{r.description || 'No description'}}</li>`;
                                }}
                            }}).join('') + '</ul>';
                    }}
                    results.classList.add('active');
                }});
        }}
        document.getElementById('search-input').addEventListener('keypress', e => {{
            if (e.key === 'Enter') doSearch();
        }});
    </script>
</body>
</html>"#,
        state.api_name,
        state.api_name,
        state.api_version,
        stats.total_endpoints,
        stats.total_schemas,
        stats.documentation_coverage,
        endpoints_html,
        schemas_html
    );

    Html(html)
}

/// OpenAPI documentation for help endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        help_index,
        list_endpoints,
        get_endpoint,
        list_schemas,
        get_schema,
        search,
        get_stats,
        help_html,
    ),
    components(schemas(
        HelpIndexResponse,
        HelpLinks,
        HelpErrorResponse,
        ApiStats,
        crate::schema::SearchResult,
        crate::schema::HttpMethod,
    )),
    tags(
        (name = "Help", description = "API documentation and help endpoints")
    )
)]
pub struct HelpApiDoc;

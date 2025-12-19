import type {
  PriceEntry,
  PriceFilter,
  CncfTool,
  CncfToolCategory,
  CncfToolsResponse,
  CncfCategoryGroup,
  CategoryRecommendationResponse,
  TcoCalculationRequest,
  TcoCalculationResult,
  InfrastructureCostComparison,
  CloudProvider,
  DeploymentMode,
  Money,
  ChatRequest,
  ChatResponse,
  ChatSession,
  CreateSession,
  ChatMessage,
} from "~/types";

const API_BASE = import.meta.env.VITE_API_URL || "/api";
// The pricing API is nested under /cloud-prices based on the entity table name
const PRICING_BASE = "/cloud-prices";

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  return response.json();
}

// Pricing API
export async function fetchPrices(filter: PriceFilter = {}): Promise<PriceEntry[]> {
  const params = new URLSearchParams();

  if (filter.provider) params.set("provider", filter.provider);
  if (filter.resource_type) params.set("resource_type", filter.resource_type);
  if (filter.regions?.length) params.set("regions", filter.regions.join(","));
  if (filter.service_name) params.set("service_name", filter.service_name);
  if (filter.instance_type) params.set("instance_type", filter.instance_type);
  if (filter.limit) params.set("limit", filter.limit.toString());
  if (filter.offset) params.set("offset", filter.offset.toString());

  const query = params.toString();
  return fetchJson<PriceEntry[]>(`${PRICING_BASE}${query ? `?${query}` : ""}`);
}

export async function fetchPriceById(id: string): Promise<PriceEntry> {
  return fetchJson<PriceEntry>(`${PRICING_BASE}/${id}`);
}

export async function fetchPriceStats(): Promise<{
  total_count: number;
  by_provider: { aws: number; azure: number; gcp: number };
}> {
  return fetchJson(`${PRICING_BASE}/stats`);
}

// CNCF Tools API
export async function fetchCncfTools(): Promise<CncfTool[]> {
  return fetchJson<CncfTool[]>(`${PRICING_BASE}/cncf/tools`);
}

export async function fetchCncfTool(toolId: string): Promise<CncfTool> {
  return fetchJson<CncfTool>(`${PRICING_BASE}/cncf/tools/${toolId}`);
}

// CNCF Landscape API (Real data from CNCF)
export async function fetchCncfLandscape(): Promise<CncfToolsResponse> {
  return fetchJson<CncfToolsResponse>(`${PRICING_BASE}/cncf/landscape`);
}

export async function fetchCncfCategory(category: CncfToolCategory): Promise<CncfCategoryGroup> {
  return fetchJson<CncfCategoryGroup>(`${PRICING_BASE}/cncf/landscape/${category}`);
}

export async function fetchCncfRecommendations(category: CncfToolCategory): Promise<CategoryRecommendationResponse> {
  return fetchJson<CategoryRecommendationResponse>(`${PRICING_BASE}/cncf/recommend/${category}`);
}

// TCO Calculator API
export async function calculateTco(
  request: TcoCalculationRequest
): Promise<TcoCalculationResult> {
  return fetchJson<TcoCalculationResult>(`${PRICING_BASE}/tco/calculate`, {
    method: "POST",
    body: JSON.stringify(request),
  });
}

export async function compareInfrastructure(params: {
  provider: CloudProvider;
  region: string;
  deployment_mode: DeploymentMode;
  engineer_hourly_rate: Money;
  workload_count: number;
}): Promise<InfrastructureCostComparison> {
  return fetchJson<InfrastructureCostComparison>(`${PRICING_BASE}/tco/compare`, {
    method: "POST",
    body: JSON.stringify(params),
  });
}

// Mock data for development (when API is not available)
export const MOCK_PRICES: PriceEntry[] = [
  {
    id: "1",
    provider: "aws",
    resource_type: "compute",
    sku: "aws-ec2-t3.micro-us-east-1",
    service_name: "Amazon EC2",
    product_family: "Compute Instance",
    instance_type: "t3.micro",
    region: "us-east-1",
    unit_price: { amount: 104, currency: "USD", decimal_places: 2 },
    pricing_unit: "hour",
    description: "t3.micro - 2 vCPU, 1 GB memory",
    attributes: { vcpu: "2", memory_gb: "1" },
    effective_date: new Date().toISOString(),
    expiration_date: null,
    collected_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "2",
    provider: "azure",
    resource_type: "compute",
    sku: "azure-vm-B1s-eastus",
    service_name: "Azure Virtual Machines",
    product_family: "Compute",
    instance_type: "B1s",
    region: "eastus",
    unit_price: { amount: 104, currency: "USD", decimal_places: 2 },
    pricing_unit: "hour",
    description: "B1s - 1 vCPU, 1 GB memory",
    attributes: { vcpu: "1", memory_gb: "1" },
    effective_date: new Date().toISOString(),
    expiration_date: null,
    collected_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "3",
    provider: "gcp",
    resource_type: "compute",
    sku: "gcp-compute-e2-micro-us-central1",
    service_name: "Compute Engine",
    product_family: "Compute",
    instance_type: "e2-micro",
    region: "us-central1",
    unit_price: { amount: 67, currency: "USD", decimal_places: 2 },
    pricing_unit: "hour",
    description: "e2-micro - 2 vCPU, 1 GB memory",
    attributes: { vcpu: "2", memory_gb: "1" },
    effective_date: new Date().toISOString(),
    expiration_date: null,
    collected_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

export const MOCK_CNCF_TOOLS: CncfTool[] = [
  {
    id: "cnpg",
    name: "CloudNativePG",
    category: "database",
    maturity: "sandbox",
    replaces_resource_type: "database",
    github_stars: 4500,
    project_url: "https://cloudnative-pg.io",
    description: "Kubernetes operator for PostgreSQL with HA, backups, and monitoring",
    operator_requirements: { cpu_millicores: 100, memory_mb: 256, storage_gb: 0, replicas: 1 },
    minimal_requirements: { cpu_millicores: 500, memory_mb: 1024, storage_gb: 20, replicas: 1 },
    ha_requirements: { cpu_millicores: 1000, memory_mb: 4096, storage_gb: 100, replicas: 3 },
    production_requirements: { cpu_millicores: 2000, memory_mb: 8192, storage_gb: 500, replicas: 3 },
    ops_hours: {
      initial_setup_hours: 16,
      minimal_monthly_hours: 2,
      ha_monthly_hours: 4,
      production_monthly_hours: 8,
    },
    managed_equivalents: [
      {
        provider: "aws",
        service_name: "Amazon RDS PostgreSQL",
        minimal_equivalent_sku: "db.t3.micro",
        ha_equivalent_sku: "db.r5.large",
        production_equivalent_sku: "db.r5.xlarge",
      },
      {
        provider: "azure",
        service_name: "Azure Database for PostgreSQL",
        minimal_equivalent_sku: "B1ms",
        ha_equivalent_sku: "D2s_v3",
        production_equivalent_sku: "D4s_v3",
      },
      {
        provider: "gcp",
        service_name: "Cloud SQL PostgreSQL",
        minimal_equivalent_sku: "db-f1-micro",
        ha_equivalent_sku: "db-n1-standard-2",
        production_equivalent_sku: "db-n1-standard-4",
      },
    ],
    included_features: [
      "High Availability (streaming replication)",
      "Automated failover",
      "Backup to S3/GCS/Azure Blob",
      "Point-in-time recovery",
      "Connection pooling (PgBouncer)",
      "Prometheus metrics",
      "Rolling updates",
    ],
  },
  {
    id: "redis-operator",
    name: "Redis Operator",
    category: "cache",
    maturity: "sandbox",
    replaces_resource_type: "database",
    github_stars: 1500,
    project_url: "https://github.com/spotahome/redis-operator",
    description: "Kubernetes operator for Redis with sentinel HA",
    operator_requirements: { cpu_millicores: 100, memory_mb: 128, storage_gb: 0, replicas: 1 },
    minimal_requirements: { cpu_millicores: 250, memory_mb: 512, storage_gb: 0, replicas: 1 },
    ha_requirements: { cpu_millicores: 500, memory_mb: 2048, storage_gb: 0, replicas: 3 },
    production_requirements: { cpu_millicores: 1000, memory_mb: 8192, storage_gb: 0, replicas: 6 },
    ops_hours: {
      initial_setup_hours: 8,
      minimal_monthly_hours: 1,
      ha_monthly_hours: 2,
      production_monthly_hours: 4,
    },
    managed_equivalents: [
      {
        provider: "aws",
        service_name: "Amazon ElastiCache Redis",
        minimal_equivalent_sku: "cache.t3.micro",
        ha_equivalent_sku: "cache.r5.large",
        production_equivalent_sku: "cache.r5.xlarge",
      },
      {
        provider: "azure",
        service_name: "Azure Cache for Redis",
        minimal_equivalent_sku: "C0-Basic",
        ha_equivalent_sku: "C1-Standard",
        production_equivalent_sku: "P1-Premium",
      },
      {
        provider: "gcp",
        service_name: "Memorystore Redis",
        minimal_equivalent_sku: "M1",
        ha_equivalent_sku: "M2-Standard",
        production_equivalent_sku: "M5-Standard",
      },
    ],
    included_features: [
      "Sentinel-based HA",
      "Automatic failover",
      "Redis Cluster support",
      "Prometheus metrics",
      "Persistent storage (optional)",
    ],
  },
  {
    id: "strimzi",
    name: "Strimzi",
    category: "message_queue",
    maturity: "incubating",
    replaces_resource_type: "other",
    github_stars: 4800,
    project_url: "https://strimzi.io",
    description: "Kubernetes operator for Apache Kafka",
    operator_requirements: { cpu_millicores: 200, memory_mb: 384, storage_gb: 0, replicas: 1 },
    minimal_requirements: { cpu_millicores: 1000, memory_mb: 2048, storage_gb: 50, replicas: 1 },
    ha_requirements: { cpu_millicores: 2000, memory_mb: 4096, storage_gb: 200, replicas: 3 },
    production_requirements: { cpu_millicores: 4000, memory_mb: 8192, storage_gb: 1000, replicas: 5 },
    ops_hours: {
      initial_setup_hours: 24,
      minimal_monthly_hours: 4,
      ha_monthly_hours: 8,
      production_monthly_hours: 16,
    },
    managed_equivalents: [
      {
        provider: "aws",
        service_name: "Amazon MSK",
        minimal_equivalent_sku: "kafka.t3.small",
        ha_equivalent_sku: "kafka.m5.large",
        production_equivalent_sku: "kafka.m5.2xlarge",
      },
      {
        provider: "azure",
        service_name: "Azure Event Hubs",
        minimal_equivalent_sku: "Basic",
        ha_equivalent_sku: "Standard",
        production_equivalent_sku: "Premium",
      },
      {
        provider: "gcp",
        service_name: "Confluent Cloud",
        minimal_equivalent_sku: "Basic",
        ha_equivalent_sku: "Standard",
        production_equivalent_sku: "Dedicated",
      },
    ],
    included_features: [
      "Kafka cluster management",
      "ZooKeeper or KRaft mode",
      "Kafka Connect",
      "Schema Registry",
      "Cruise Control (rebalancing)",
      "TLS encryption",
      "SASL authentication",
      "Prometheus metrics",
    ],
  },
];

// ===== FinOps Chat API =====

const FINOPS_BASE = "/finops";

// Send a chat message (non-streaming)
export async function sendChatMessage(request: ChatRequest): Promise<ChatResponse> {
  return fetchJson<ChatResponse>(`${FINOPS_BASE}/chat`, {
    method: "POST",
    body: JSON.stringify(request),
  });
}

// Stream a chat response via SSE
export function streamChatMessage(
  request: ChatRequest,
  onMessage: (event: string, data: string) => void,
  onError?: (error: Error) => void,
  onComplete?: () => void
): () => void {
  const params = new URLSearchParams({
    message: request.message,
  });

  if (request.session_id) {
    params.set("session_id", request.session_id);
  }

  if (request.user_id) {
    params.set("user_id", request.user_id);
  }

  if (request.context) {
    params.set("context", JSON.stringify(request.context));
  }

  const eventSource = new EventSource(`${API_BASE}${FINOPS_BASE}/chat/stream?${params}`);

  eventSource.onmessage = (event) => {
    onMessage("message", event.data);
  };

  eventSource.addEventListener("text", (event) => {
    onMessage("text", (event as MessageEvent).data);
  });

  eventSource.addEventListener("tool_call", (event) => {
    onMessage("tool_call", (event as MessageEvent).data);
  });

  eventSource.addEventListener("tool_result", (event) => {
    onMessage("tool_result", (event as MessageEvent).data);
  });

  eventSource.addEventListener("done", (event) => {
    onMessage("done", (event as MessageEvent).data);
    eventSource.close();
    onComplete?.();
  });

  eventSource.addEventListener("error", (event) => {
    const errorEvent = event as MessageEvent;
    if (errorEvent.data) {
      onMessage("error", errorEvent.data);
    }
    onError?.(new Error("SSE connection error"));
    eventSource.close();
  });

  eventSource.onerror = () => {
    onError?.(new Error("SSE connection failed"));
    eventSource.close();
  };

  // Return cleanup function
  return () => {
    eventSource.close();
  };
}

// Session management
export async function createSession(request: CreateSession): Promise<ChatSession> {
  return fetchJson<ChatSession>(`${FINOPS_BASE}/sessions`, {
    method: "POST",
    body: JSON.stringify(request),
  });
}

export async function fetchSessions(userId?: string): Promise<ChatSession[]> {
  const params = userId ? `?user_id=${userId}` : "";
  return fetchJson<ChatSession[]>(`${FINOPS_BASE}/sessions${params}`);
}

export async function fetchSession(sessionId: string): Promise<ChatSession> {
  return fetchJson<ChatSession>(`${FINOPS_BASE}/sessions/${sessionId}`);
}

export async function deleteSession(sessionId: string): Promise<void> {
  await fetch(`${API_BASE}${FINOPS_BASE}/sessions/${sessionId}`, {
    method: "DELETE",
  });
}

// Message history
export async function fetchSessionMessages(sessionId: string): Promise<ChatMessage[]> {
  return fetchJson<ChatMessage[]>(`${FINOPS_BASE}/sessions/${sessionId}/messages`);
}

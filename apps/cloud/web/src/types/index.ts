// Cloud Provider Types
export type CloudProvider = "aws" | "azure" | "gcp";

export type ResourceType =
  | "compute"
  | "storage"
  | "database"
  | "network"
  | "serverless"
  | "analytics"
  | "kubernetes"
  | "other";

export type PricingUnit =
  | "hour"
  | "month"
  | "gb"
  | "gb_hour"
  | "gb_month"
  | "request"
  | "million_requests"
  | "second"
  | "unit";

export type Currency = "USD" | "EUR" | "GBP";

// Money representation
export interface Money {
  amount: number; // In cents
  currency: Currency;
  decimal_places: number;
}

// Price entry from cloud providers
export interface PriceEntry {
  id: string;
  provider: CloudProvider;
  resource_type: ResourceType;
  sku: string;
  service_name: string;
  product_family: string;
  instance_type: string | null;
  region: string;
  unit_price: Money;
  pricing_unit: PricingUnit;
  description: string;
  attributes: Record<string, string>;
  effective_date: string;
  expiration_date: string | null;
  collected_at: string;
  created_at: string;
  updated_at: string;
}

// Filter for querying prices
export interface PriceFilter {
  provider?: CloudProvider;
  resource_type?: ResourceType;
  regions?: string[];
  service_name?: string;
  instance_type?: string;
  sku?: string;
  limit?: number;
  offset?: number;
}

// CNCF Tool Types
export type CncfToolCategory =
  | "database"
  | "cache"
  | "message_queue"
  | "storage"
  | "observability"
  | "service_mesh"
  | "gitops";

export type CncfMaturity = "sandbox" | "incubating" | "graduated";

export type DeploymentMode = "minimal" | "high_availability" | "production";

export interface ResourceRequirements {
  cpu_millicores: number;
  memory_mb: number;
  storage_gb: number;
  replicas: number;
}

export interface OpsHoursEstimate {
  initial_setup_hours: number;
  minimal_monthly_hours: number;
  ha_monthly_hours: number;
  production_monthly_hours: number;
}

export interface ManagedServiceEquivalent {
  provider: CloudProvider;
  service_name: string;
  minimal_equivalent_sku: string;
  ha_equivalent_sku: string;
  production_equivalent_sku: string;
}

export interface CncfTool {
  id: string;
  name: string;
  category: CncfToolCategory;
  maturity: CncfMaturity;
  replaces_resource_type: ResourceType;
  github_stars: number | null;
  project_url: string;
  description: string;
  operator_requirements: ResourceRequirements;
  minimal_requirements: ResourceRequirements;
  ha_requirements: ResourceRequirements;
  production_requirements: ResourceRequirements;
  ops_hours: OpsHoursEstimate;
  managed_equivalents: ManagedServiceEquivalent[];
  included_features: string[];
}

// TCO Calculation Types
export interface TcoCalculationRequest {
  tool_id: string;
  deployment_mode: DeploymentMode;
  provider: CloudProvider;
  region: string;
  engineer_hourly_rate: Money;
  include_control_plane: boolean;
  workload_count: number;
}

export type CostRecommendation =
  | "strongly_self_managed"
  | "consider_self_managed"
  | "similar"
  | "consider_managed"
  | "strongly_managed";

export interface TcoCalculationResult {
  tool_id: string;
  tool_name: string;
  deployment_mode: DeploymentMode;
  provider: CloudProvider;
  region: string;
  control_plane_cost: Money;
  operator_compute_cost: Money;
  workload_compute_cost: Money;
  storage_cost: Money;
  backup_storage_cost: Money;
  total_infra_cost: Money;
  ops_hours_per_month: number;
  ops_cost: Money;
  amortized_ops_cost: Money;
  total_self_managed_cost: Money;
  managed_service_name: string;
  managed_service_sku: string;
  managed_service_cost: Money;
  savings_vs_managed: Money;
  percentage_difference: number;
  break_even_ops_hours: number;
  recommendation: CostRecommendation;
}

export interface InfrastructureCostComparison {
  all_managed_cost: Money;
  all_self_managed_cost: Money;
  hybrid_cost: Money;
  tool_comparisons: TcoCalculationResult[];
  recommendations: Record<string, CostRecommendation>;
}

// CNCF Landscape Types (Real data from CNCF)
export interface GitHubStats {
  stars: number;
  forks: number;
  open_issues: number;
  last_commit: string;
  contributors: number;
}

export interface CncfToolEnriched {
  id: string;
  name: string;
  category: CncfToolCategory;
  subcategory: string | null;
  maturity: CncfMaturity;
  project_url: string;
  repo_url: string | null;
  description: string;
  logo_url: string | null;
  github_stats: GitHubStats | null;
  pros: string[];
  cons: string[];
  recommendation_score: number | null;
  updated_at: string;
}

export interface CncfCategoryGroup {
  category: CncfToolCategory;
  tools: CncfToolEnriched[];
  total: number;
}

export interface CncfToolsResponse {
  categories: CncfCategoryGroup[];
  total_tools: number;
  last_updated: string;
}

export interface ToolRecommendation {
  tool_id: string;
  tool_name: string;
  pros: string[];
  cons: string[];
  best_for: string[];
  avoid_if: string[];
  score: number;
}

export interface CategoryRecommendationResponse {
  category: CncfToolCategory;
  recommendations: ToolRecommendation[];
  top_pick: string;
  top_pick_reason: string;
}

// Helper functions
export function formatMoney(money: Money): string {
  const value = money.amount / Math.pow(10, money.decimal_places);
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: money.currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

export function formatMoneyPerMonth(money: Money): string {
  return `${formatMoney(money)}/mo`;
}

export function formatMoneyPerHour(money: Money): string {
  return `${formatMoney(money)}/hr`;
}

export function getProviderColor(provider: CloudProvider): string {
  switch (provider) {
    case "aws":
      return "#FF9900";
    case "azure":
      return "#0078D4";
    case "gcp":
      return "#4285F4";
  }
}

export function getProviderName(provider: CloudProvider): string {
  switch (provider) {
    case "aws":
      return "AWS";
    case "azure":
      return "Azure";
    case "gcp":
      return "GCP";
  }
}

export function getRecommendationText(rec: CostRecommendation): string {
  switch (rec) {
    case "strongly_self_managed":
      return "Strongly Recommend Self-Managed";
    case "consider_self_managed":
      return "Consider Self-Managed";
    case "similar":
      return "Similar Cost";
    case "consider_managed":
      return "Consider Managed";
    case "strongly_managed":
      return "Strongly Recommend Managed";
  }
}

export function getRecommendationColor(rec: CostRecommendation): string {
  switch (rec) {
    case "strongly_self_managed":
      return "text-green-600";
    case "consider_self_managed":
      return "text-green-500";
    case "similar":
      return "text-yellow-500";
    case "consider_managed":
      return "text-orange-500";
    case "strongly_managed":
      return "text-red-500";
  }
}

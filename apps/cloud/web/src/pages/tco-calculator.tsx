import { Component, createSignal, createMemo, Show, For } from "solid-js";
import { createQuery, createMutation } from "@tanstack/solid-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Select } from "~/components/ui/select";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Badge } from "~/components/ui/badge";
import { ProviderBadge } from "~/components/provider-badge";
import { fetchCncfTools, calculateTco, MOCK_CNCF_TOOLS } from "~/lib/api-client";
import {
  type CloudProvider,
  type DeploymentMode,
  type CncfTool,
  type TcoCalculationResult,
  type Money,
  formatMoney,
  formatMoneyPerMonth,
  getRecommendationText,
  getRecommendationColor,
} from "~/types";

// Simple TCO calculation (in production, this would call the API)
function calculateTcoLocally(
  tool: CncfTool,
  provider: CloudProvider,
  deploymentMode: DeploymentMode,
  engineerRate: number,
  workloadCount: number
): TcoCalculationResult {
  const reqs =
    deploymentMode === "minimal"
      ? tool.minimal_requirements
      : deploymentMode === "high_availability"
      ? tool.ha_requirements
      : tool.production_requirements;

  const opsHours =
    deploymentMode === "minimal"
      ? tool.ops_hours.minimal_monthly_hours
      : deploymentMode === "high_availability"
      ? tool.ops_hours.ha_monthly_hours
      : tool.ops_hours.production_monthly_hours;

  // Control plane cost (monthly)
  const controlPlaneCost = provider === "azure" ? 0 : 7300; // AKS is free!

  // Compute cost estimate (simplified)
  const cpuCost = (reqs.cpu_millicores / 1000) * reqs.replicas * 50 * 100; // $50/vCPU/mo
  const memoryCost = (reqs.memory_mb / 1024) * reqs.replicas * 7 * 100; // $7/GB/mo
  const operatorCost = (tool.operator_requirements.cpu_millicores / 1000) * 50 * 100;
  const storageCost = reqs.storage_gb * reqs.replicas * 10; // $0.10/GB/mo

  const totalInfraCost = controlPlaneCost + cpuCost + memoryCost + operatorCost + storageCost;
  const opsCost = opsHours * engineerRate * 100; // Convert to cents
  const amortizedOpsCost = opsCost / Math.max(workloadCount, 1);
  const totalSelfManagedCost = totalInfraCost + amortizedOpsCost;

  // Managed service cost estimate
  const managedCostBase =
    deploymentMode === "minimal" ? 5000 : deploymentMode === "high_availability" ? 20000 : 50000;
  const managedServiceCost = managedCostBase * (provider === "gcp" ? 0.9 : provider === "azure" ? 0.95 : 1);

  const savings = managedServiceCost - totalSelfManagedCost;
  const percentageDiff = managedServiceCost > 0 ? (savings / managedServiceCost) * 100 : 0;

  let recommendation: TcoCalculationResult["recommendation"];
  if (percentageDiff > 30) recommendation = "strongly_self_managed";
  else if (percentageDiff > 10) recommendation = "consider_self_managed";
  else if (percentageDiff > -10) recommendation = "similar";
  else if (percentageDiff > -30) recommendation = "consider_managed";
  else recommendation = "strongly_managed";

  const makeMoney = (amount: number): Money => ({ amount: Math.round(amount), currency: "USD", decimal_places: 2 });

  return {
    tool_id: tool.id,
    tool_name: tool.name,
    deployment_mode: deploymentMode,
    provider,
    region: "us-east-1",
    control_plane_cost: makeMoney(controlPlaneCost),
    operator_compute_cost: makeMoney(operatorCost),
    workload_compute_cost: makeMoney(cpuCost + memoryCost),
    storage_cost: makeMoney(storageCost),
    backup_storage_cost: makeMoney(storageCost / 2),
    total_infra_cost: makeMoney(totalInfraCost),
    ops_hours_per_month: opsHours,
    ops_cost: makeMoney(opsCost),
    amortized_ops_cost: makeMoney(amortizedOpsCost),
    total_self_managed_cost: makeMoney(totalSelfManagedCost),
    managed_service_name: tool.managed_equivalents.find((e) => e.provider === provider)?.service_name || "Unknown",
    managed_service_sku:
      deploymentMode === "minimal"
        ? tool.managed_equivalents.find((e) => e.provider === provider)?.minimal_equivalent_sku || ""
        : deploymentMode === "high_availability"
        ? tool.managed_equivalents.find((e) => e.provider === provider)?.ha_equivalent_sku || ""
        : tool.managed_equivalents.find((e) => e.provider === provider)?.production_equivalent_sku || "",
    managed_service_cost: makeMoney(managedServiceCost),
    savings_vs_managed: makeMoney(savings),
    percentage_difference: percentageDiff,
    break_even_ops_hours: totalInfraCost < managedServiceCost ? (managedServiceCost - totalInfraCost) / (engineerRate * 100) : 0,
    recommendation,
  };
}

const TcoCalculator: Component = () => {
  const [selectedTool, setSelectedTool] = createSignal<string>("cnpg");
  const [selectedProvider, setSelectedProvider] = createSignal<CloudProvider>("aws");
  const [deploymentMode, setDeploymentMode] = createSignal<DeploymentMode>("high_availability");
  const [engineerRate, setEngineerRate] = createSignal(150);
  const [workloadCount, setWorkloadCount] = createSignal(1);
  const [result, setResult] = createSignal<TcoCalculationResult | null>(null);

  // Fetch CNCF tools from API (falls back to mock if API unavailable)
  const toolsQuery = createQuery(() => ({
    queryKey: ["cncf-tools"],
    queryFn: fetchCncfTools,
    staleTime: 1000 * 60 * 10,
    retry: 1,
    placeholderData: MOCK_CNCF_TOOLS,
  }));

  // Mutation for TCO calculation
  const tcoMutation = createMutation(() => ({
    mutationFn: calculateTco,
    onSuccess: (data) => setResult(data),
    onError: () => {
      // Fall back to local calculation if API fails
      const tool = currentTool();
      if (tool) {
        const calculation = calculateTcoLocally(
          tool,
          selectedProvider(),
          deploymentMode(),
          engineerRate(),
          workloadCount()
        );
        setResult(calculation);
      }
    },
  }));

  const tools = () => toolsQuery.data ?? MOCK_CNCF_TOOLS;
  const currentTool = createMemo(() => tools().find((t) => t.id === selectedTool()));

  const handleCalculate = () => {
    const tool = currentTool();
    if (!tool) return;

    // Try API first, fall back to local calculation
    tcoMutation.mutate({
      tool_id: tool.id,
      provider: selectedProvider(),
      region: "us-east-1",
      deployment_mode: deploymentMode(),
      engineer_hourly_rate: { amount: engineerRate() * 100, currency: "USD", decimal_places: 2 },
      workload_count: workloadCount(),
    });
  };

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">TCO Calculator</h1>
        <p class="text-muted-foreground mt-2">
          Compare Total Cost of Ownership: Managed Services vs Self-Managed CNCF Tools
        </p>
      </div>

      <div class="grid gap-8 lg:grid-cols-2">
        {/* Input Form */}
        <Card>
          <CardHeader>
            <CardTitle>Configuration</CardTitle>
            <CardDescription>Select your tool, provider, and deployment parameters</CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <div class="space-y-2">
              <Label for="tool">CNCF Tool</Label>
              <Select
                id="tool"
                value={selectedTool()}
                onChange={(e) => setSelectedTool(e.currentTarget.value)}
              >
                <For each={tools()}>
                  {(tool) => (
                    <option value={tool.id}>
                      {tool.name} - {tool.category}
                    </option>
                  )}
                </For>
              </Select>
              <Show when={currentTool()}>
                <p class="text-xs text-muted-foreground">{currentTool()!.description}</p>
              </Show>
            </div>

            <div class="space-y-2">
              <Label for="provider">Cloud Provider</Label>
              <Select
                id="provider"
                value={selectedProvider()}
                onChange={(e) => setSelectedProvider(e.currentTarget.value as CloudProvider)}
              >
                <option value="aws">AWS</option>
                <option value="azure">Azure (Free K8s Control Plane!)</option>
                <option value="gcp">GCP</option>
              </Select>
            </div>

            <div class="space-y-2">
              <Label for="deployment">Deployment Mode</Label>
              <Select
                id="deployment"
                value={deploymentMode()}
                onChange={(e) => setDeploymentMode(e.currentTarget.value as DeploymentMode)}
              >
                <option value="minimal">Minimal (Dev/Test)</option>
                <option value="high_availability">High Availability (Staging)</option>
                <option value="production">Production (Full Features)</option>
              </Select>
            </div>

            <div class="space-y-2">
              <Label for="engineer-rate">Engineer Hourly Rate ($)</Label>
              <Input
                id="engineer-rate"
                type="number"
                min={0}
                value={engineerRate()}
                onInput={(e) => setEngineerRate(parseInt(e.currentTarget.value) || 0)}
              />
              <p class="text-xs text-muted-foreground">
                Used to calculate operational cost for self-managed
              </p>
            </div>

            <div class="space-y-2">
              <Label for="workload-count">Similar Workloads</Label>
              <Input
                id="workload-count"
                type="number"
                min={1}
                value={workloadCount()}
                onInput={(e) => setWorkloadCount(parseInt(e.currentTarget.value) || 1)}
              />
              <p class="text-xs text-muted-foreground">
                Ops cost is amortized across multiple similar workloads
              </p>
            </div>

            <Button class="w-full" onClick={handleCalculate} disabled={tcoMutation.isPending}>
              {tcoMutation.isPending ? "Calculating..." : "Calculate TCO"}
            </Button>
          </CardContent>
        </Card>

        {/* Results */}
        <Show when={result()}>
          {(res) => (
            <div class="space-y-4">
              {/* Recommendation Card */}
              <Card>
                <CardHeader>
                  <CardTitle class="flex items-center justify-between">
                    Recommendation
                    <Badge
                      variant={
                        res().recommendation.includes("self_managed") ? "success" :
                        res().recommendation === "similar" ? "warning" : "destructive"
                      }
                    >
                      {res().percentage_difference > 0 ? "Save" : "Extra"}{" "}
                      {Math.abs(res().percentage_difference).toFixed(0)}%
                    </Badge>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <p class={`text-lg font-semibold ${getRecommendationColor(res().recommendation)}`}>
                    {getRecommendationText(res().recommendation)}
                  </p>
                  <p class="text-sm text-muted-foreground mt-2">
                    {res().break_even_ops_hours > 0
                      ? `Break-even at ${res().break_even_ops_hours.toFixed(1)} ops hours/month`
                      : "Self-managed is more expensive even with zero ops time"}
                  </p>
                </CardContent>
              </Card>

              {/* Cost Comparison */}
              <Card>
                <CardHeader>
                  <CardTitle>Cost Comparison (Monthly)</CardTitle>
                </CardHeader>
                <CardContent>
                  <div class="grid grid-cols-2 gap-4">
                    {/* Self-Managed */}
                    <div class="space-y-3 p-4 bg-muted/50 rounded-lg">
                      <h4 class="font-semibold text-sm">Self-Managed ({res().tool_name})</h4>
                      <div class="space-y-1 text-sm">
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Control Plane</span>
                          <span>{formatMoney(res().control_plane_cost)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Operator</span>
                          <span>{formatMoney(res().operator_compute_cost)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Workload Compute</span>
                          <span>{formatMoney(res().workload_compute_cost)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Storage</span>
                          <span>{formatMoney(res().storage_cost)}</span>
                        </div>
                        <div class="flex justify-between border-t pt-1">
                          <span class="text-muted-foreground">Infrastructure</span>
                          <span class="font-medium">{formatMoney(res().total_infra_cost)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">
                            Ops ({res().ops_hours_per_month}h @ ${engineerRate()})
                          </span>
                          <span>{formatMoney(res().amortized_ops_cost)}</span>
                        </div>
                        <div class="flex justify-between border-t pt-1 text-lg font-bold">
                          <span>Total</span>
                          <span>{formatMoneyPerMonth(res().total_self_managed_cost)}</span>
                        </div>
                      </div>
                    </div>

                    {/* Managed Service */}
                    <div class="space-y-3 p-4 bg-muted/50 rounded-lg">
                      <h4 class="font-semibold text-sm">Managed Service</h4>
                      <div class="space-y-1 text-sm">
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Service</span>
                          <span>{res().managed_service_name}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">SKU</span>
                          <code class="text-xs bg-background px-1 rounded">
                            {res().managed_service_sku}
                          </code>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-muted-foreground">Provider</span>
                          <ProviderBadge provider={res().provider} />
                        </div>
                        <div class="pt-8" />
                        <div class="flex justify-between border-t pt-1 text-lg font-bold">
                          <span>Total</span>
                          <span>{formatMoneyPerMonth(res().managed_service_cost)}</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Savings */}
                  <div class="mt-4 p-4 bg-primary/10 rounded-lg text-center">
                    <p class="text-sm text-muted-foreground">
                      {res().savings_vs_managed.amount > 0 ? "Potential Savings" : "Additional Cost"}
                    </p>
                    <p class="text-2xl font-bold">
                      {res().savings_vs_managed.amount > 0 ? "+" : ""}
                      {formatMoneyPerMonth(res().savings_vs_managed)}
                    </p>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}
        </Show>

        <Show when={!result()}>
          <Card class="flex items-center justify-center min-h-[400px]">
            <CardContent class="text-center text-muted-foreground">
              <p>Configure your parameters and click Calculate to see results</p>
            </CardContent>
          </Card>
        </Show>
      </div>
    </div>
  );
};

export default TcoCalculator;

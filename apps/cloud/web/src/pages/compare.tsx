import { Component, createSignal, createMemo, For, Show } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Select } from "~/components/ui/select";
import { Badge } from "~/components/ui/badge";
import { ProviderBadge } from "~/components/provider-badge";
import { fetchPrices, MOCK_PRICES } from "~/lib/api-client";
import {
  type CloudProvider,
  type ResourceType,
  type PriceEntry,
  formatMoney,
  formatMoneyPerHour,
  getProviderName,
} from "~/types";

interface ComparisonGroup {
  key: string;
  description: string;
  prices: PriceEntry[];
  cheapest: PriceEntry | null;
}

const Compare: Component = () => {
  const [selectedResourceType, setSelectedResourceType] = createSignal<ResourceType>("compute");
  const [selectedMetric, setSelectedMetric] = createSignal<"vcpu" | "memory" | "storage">("vcpu");

  // Fetch prices from API (falls back to mock if API unavailable)
  const pricesQuery = createQuery(() => ({
    queryKey: ["prices"],
    queryFn: () => fetchPrices({ limit: 500 }),
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: 1,
    placeholderData: MOCK_PRICES,
  }));

  const prices = () => pricesQuery.data ?? MOCK_PRICES;

  // Group prices by comparable specs
  const comparisonGroups = createMemo((): ComparisonGroup[] => {
    const filtered = prices().filter((p) => p.resource_type === selectedResourceType());

    // Group by similar specs (simplified - in production would be more sophisticated)
    const groups = new Map<string, PriceEntry[]>();

    for (const price of filtered) {
      let key: string;
      const vcpu = price.attributes.vcpu || "unknown";
      const memory = price.attributes.memory_gb || price.attributes.memory || "unknown";

      if (selectedResourceType() === "compute") {
        key = `${vcpu}-vcpu-${memory}-gb`;
      } else if (selectedResourceType() === "storage") {
        key = price.instance_type || price.attributes.storage_class || "standard";
      } else if (selectedResourceType() === "database") {
        key = `${vcpu}-vcpu-${memory}-gb-db`;
      } else if (selectedResourceType() === "kubernetes") {
        key = price.instance_type || "cluster";
      } else {
        key = price.service_name;
      }

      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(price);
    }

    // Convert to comparison groups
    return Array.from(groups.entries())
      .map(([key, groupPrices]) => {
        const sorted = [...groupPrices].sort(
          (a, b) => a.unit_price.amount - b.unit_price.amount
        );
        return {
          key,
          description: groupPrices[0]?.description || key,
          prices: sorted,
          cheapest: sorted[0] || null,
        };
      })
      .filter((g) => g.prices.length > 0);
  });

  // Calculate savings summary
  const savingsSummary = createMemo(() => {
    const groups = comparisonGroups();
    let totalPotentialSavings = 0;
    let comparisonCount = 0;

    const providerWins = { aws: 0, azure: 0, gcp: 0 };

    for (const group of groups) {
      if (group.prices.length > 1 && group.cheapest) {
        const cheapestPrice = group.cheapest.unit_price.amount;
        const mostExpensive = Math.max(...group.prices.map((p) => p.unit_price.amount));
        totalPotentialSavings += mostExpensive - cheapestPrice;
        comparisonCount++;
        providerWins[group.cheapest.provider]++;
      }
    }

    return {
      totalPotentialSavings,
      comparisonCount,
      providerWins,
      winningProvider: Object.entries(providerWins).sort((a, b) => b[1] - a[1])[0]?.[0] as
        | CloudProvider
        | undefined,
    };
  });

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Price Comparison</h1>
        <p class="text-muted-foreground mt-2">
          Compare equivalent services across AWS, Azure, and GCP
        </p>
      </div>

      {/* Summary Cards */}
      <div class="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">Comparisons</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{savingsSummary().comparisonCount}</div>
            <p class="text-xs text-muted-foreground">Multi-provider groups</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">AWS Wins</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-[#FF9900]">
              {savingsSummary().providerWins.aws}
            </div>
            <p class="text-xs text-muted-foreground">Cheapest option</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">Azure Wins</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-[#0078D4]">
              {savingsSummary().providerWins.azure}
            </div>
            <p class="text-xs text-muted-foreground">Cheapest option</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">GCP Wins</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-[#4285F4]">
              {savingsSummary().providerWins.gcp}
            </div>
            <p class="text-xs text-muted-foreground">Cheapest option</p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Compare By</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex gap-4">
            <div class="space-y-2">
              <label class="text-sm font-medium">Resource Type</label>
              <Select
                value={selectedResourceType()}
                onChange={(e) => setSelectedResourceType(e.currentTarget.value as ResourceType)}
              >
                <option value="compute">Compute (VMs)</option>
                <option value="storage">Storage</option>
                <option value="database">Database</option>
                <option value="kubernetes">Kubernetes</option>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Comparison Results */}
      <div class="space-y-4">
        <For each={comparisonGroups()}>
          {(group) => (
            <Card>
              <CardHeader>
                <div class="flex items-center justify-between">
                  <div>
                    <CardTitle class="text-lg">{group.key}</CardTitle>
                    <CardDescription>{group.description}</CardDescription>
                  </div>
                  <Show when={group.cheapest}>
                    <Badge variant={group.cheapest!.provider}>
                      Cheapest: {getProviderName(group.cheapest!.provider)}
                    </Badge>
                  </Show>
                </div>
              </CardHeader>
              <CardContent>
                <div class="grid gap-4 md:grid-cols-3">
                  <For each={group.prices}>
                    {(price, index) => (
                      <div
                        class={`p-4 rounded-lg border ${
                          index() === 0
                            ? "border-green-500 bg-green-50 dark:bg-green-950"
                            : "border-border"
                        }`}
                      >
                        <div class="flex items-center justify-between mb-3">
                          <ProviderBadge provider={price.provider} />
                          <Show when={index() === 0}>
                            <Badge variant="success">Best Price</Badge>
                          </Show>
                        </div>
                        <div class="space-y-2">
                          <div class="font-medium">{price.service_name}</div>
                          <div class="text-sm text-muted-foreground">
                            <code class="bg-muted px-1 py-0.5 rounded">
                              {price.instance_type || "N/A"}
                            </code>
                          </div>
                          <div class="text-2xl font-bold">
                            {formatMoneyPerHour(price.unit_price)}
                          </div>
                          <div class="text-sm text-muted-foreground">{price.region}</div>
                          <Show when={index() > 0 && group.cheapest}>
                            <div class="text-sm text-red-500">
                              +{formatMoney({
                                amount: price.unit_price.amount - group.cheapest!.unit_price.amount,
                                currency: "USD",
                                decimal_places: 2,
                              })}{" "}
                              more/hr (
                              {(
                                ((price.unit_price.amount - group.cheapest!.unit_price.amount) /
                                  group.cheapest!.unit_price.amount) *
                                100
                              ).toFixed(0)}
                              % more expensive)
                            </div>
                          </Show>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </CardContent>
            </Card>
          )}
        </For>
      </div>

      <Show when={comparisonGroups().length === 0}>
        <Card>
          <CardContent class="py-8 text-center text-muted-foreground">
            <p>No comparable prices found for {selectedResourceType()}</p>
            <p class="text-sm mt-2">Try selecting a different resource type</p>
          </CardContent>
        </Card>
      </Show>

      {/* Key Insights */}
      <Card>
        <CardHeader>
          <CardTitle>Key Insights</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid gap-4 md:grid-cols-2">
            <div class="p-4 bg-muted/50 rounded-lg">
              <h4 class="font-semibold mb-2">Kubernetes</h4>
              <p class="text-sm text-muted-foreground">
                <strong>Azure AKS</strong> offers free control plane, saving ~$73/month per cluster
                compared to AWS EKS and GCP GKE.
              </p>
            </div>
            <div class="p-4 bg-muted/50 rounded-lg">
              <h4 class="font-semibold mb-2">PostgreSQL</h4>
              <p class="text-sm text-muted-foreground">
                <strong>GCP Cloud SQL</strong> is typically 20-30% cheaper than AWS RDS and Azure
                Database for PostgreSQL at comparable tiers.
              </p>
            </div>
            <div class="p-4 bg-muted/50 rounded-lg">
              <h4 class="font-semibold mb-2">Redis/Cache</h4>
              <p class="text-sm text-muted-foreground">
                <strong>AWS ElastiCache</strong> with t3 instances offers the best value for
                smaller workloads. GCP Memorystore is most expensive.
              </p>
            </div>
            <div class="p-4 bg-muted/50 rounded-lg">
              <h4 class="font-semibold mb-2">Storage</h4>
              <p class="text-sm text-muted-foreground">
                <strong>Azure Archive</strong> storage is cheapest for cold data. For frequent
                access, all three providers are similarly priced.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Compare;

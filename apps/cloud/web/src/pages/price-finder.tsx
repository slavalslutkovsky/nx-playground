import { Component, createSignal, createResource, Show, For } from "solid-js";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Select } from "~/components/ui/select";
import { Label } from "~/components/ui/label";
import { Badge } from "~/components/ui/badge";
import { ProviderBadge } from "~/components/provider-badge";
import { fetchPrices } from "~/lib/api-client";
import {
  type CloudProvider,
  type ResourceType,
  type PriceEntry,
  formatMoneyPerHour,
  formatMoney,
  getProviderName,
} from "~/types";

// Normalized regions mapped to provider-specific regions
const REGION_MAPPINGS: Record<string, { aws: string; azure: string; gcp: string; label: string }> = {
  "us-east": {
    aws: "us-east-1",
    azure: "eastus",
    gcp: "us-east1",
    label: "US East",
  },
  "us-west": {
    aws: "us-west-2",
    azure: "westus2",
    gcp: "us-west1",
    label: "US West",
  },
  "europe-west": {
    aws: "eu-west-1",
    azure: "westeurope",
    gcp: "europe-west1",
    label: "Europe West",
  },
  "europe-north": {
    aws: "eu-north-1",
    azure: "northeurope",
    gcp: "europe-north1",
    label: "Europe North",
  },
  "asia-east": {
    aws: "ap-northeast-1",
    azure: "japaneast",
    gcp: "asia-east1",
    label: "Asia East (Tokyo)",
  },
  "asia-southeast": {
    aws: "ap-southeast-1",
    azure: "southeastasia",
    gcp: "asia-southeast1",
    label: "Asia Southeast (Singapore)",
  },
};

interface ComparisonResult {
  winner: CloudProvider;
  prices: {
    provider: CloudProvider;
    entry: PriceEntry | null;
    price: number;
    region: string;
  }[];
  savings: number;
  savingsPercent: number;
}

const PriceFinder: Component = () => {
  const [resourceType, setResourceType] = createSignal<ResourceType>("compute");
  const [region, setRegion] = createSignal<string>("us-east");
  const [serviceFilter, setServiceFilter] = createSignal<string>("all");
  const [isSearching, setIsSearching] = createSignal(false);
  const [result, setResult] = createSignal<ComparisonResult | null>(null);

  // Service options based on resource type
  const serviceOptions = () => {
    switch (resourceType()) {
      case "compute":
        return [
          { value: "all", label: "Any Compute" },
          { value: "ec2", label: "VMs (EC2/VM/Compute Engine)" },
        ];
      case "database":
        return [
          { value: "all", label: "Any Database" },
          { value: "postgresql", label: "PostgreSQL" },
          { value: "redis", label: "Redis/Cache" },
        ];
      case "kubernetes":
        return [
          { value: "all", label: "Any Kubernetes" },
          { value: "cluster", label: "K8s Control Plane" },
        ];
      case "storage":
        return [
          { value: "all", label: "Any Storage" },
          { value: "standard", label: "Standard Storage" },
        ];
      default:
        return [{ value: "all", label: "All" }];
    }
  };

  const findCheapest = async () => {
    setIsSearching(true);
    setResult(null);

    const regionMapping = REGION_MAPPINGS[region()];
    const providers: CloudProvider[] = ["aws", "azure", "gcp"];

    try {
      // Fetch prices for each provider's region
      const pricePromises = providers.map(async (provider) => {
        const providerRegion = regionMapping[provider];
        const prices = await fetchPrices({
          provider,
          resource_type: resourceType(),
          regions: [providerRegion],
          limit: 50,
        });

        // Filter by service if specified
        let filtered = prices;
        if (serviceFilter() !== "all") {
          const filter = serviceFilter().toLowerCase();
          filtered = prices.filter(
            (p) =>
              p.service_name.toLowerCase().includes(filter) ||
              p.instance_type?.toLowerCase().includes(filter) ||
              p.description.toLowerCase().includes(filter)
          );
        }

        // Find the cheapest entry for this provider
        const cheapest = filtered.reduce<PriceEntry | null>((min, entry) => {
          if (!min) return entry;
          return entry.unit_price.amount < min.unit_price.amount ? entry : min;
        }, null);

        return {
          provider,
          entry: cheapest,
          price: cheapest?.unit_price.amount ?? Infinity,
          region: providerRegion,
        };
      });

      const results = await Promise.all(pricePromises);

      // Sort by price
      const sorted = results.sort((a, b) => a.price - b.price);
      const winner = sorted[0];
      const runnerUp = sorted[1];

      // Calculate savings
      const savings = runnerUp && winner.price < Infinity
        ? runnerUp.price - winner.price
        : 0;
      const savingsPercent = runnerUp && runnerUp.price > 0 && winner.price < Infinity
        ? ((runnerUp.price - winner.price) / runnerUp.price) * 100
        : 0;

      setResult({
        winner: winner.provider,
        prices: sorted,
        savings,
        savingsPercent,
      });
    } catch (error) {
      console.error("Error fetching prices:", error);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Price Finder</h1>
        <p class="text-muted-foreground mt-2">
          Find the cheapest cloud provider for your workload
        </p>
      </div>

      <div class="grid gap-8 lg:grid-cols-2">
        {/* Search Form */}
        <Card>
          <CardHeader>
            <CardTitle>Find Cheapest Provider</CardTitle>
            <CardDescription>
              Select your requirements to compare prices across AWS, Azure, and GCP
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <div class="space-y-2">
              <Label for="resource-type">Resource Type</Label>
              <Select
                id="resource-type"
                value={resourceType()}
                onChange={(e) => {
                  setResourceType(e.currentTarget.value as ResourceType);
                  setServiceFilter("all");
                  setResult(null);
                }}
              >
                <option value="compute">Compute (VMs)</option>
                <option value="database">Database</option>
                <option value="kubernetes">Kubernetes</option>
                <option value="storage">Storage</option>
              </Select>
            </div>

            <div class="space-y-2">
              <Label for="region">Region</Label>
              <Select
                id="region"
                value={region()}
                onChange={(e) => {
                  setRegion(e.currentTarget.value);
                  setResult(null);
                }}
              >
                <For each={Object.entries(REGION_MAPPINGS)}>
                  {([key, mapping]) => (
                    <option value={key}>{mapping.label}</option>
                  )}
                </For>
              </Select>
              <p class="text-xs text-muted-foreground">
                Maps to: AWS {REGION_MAPPINGS[region()].aws}, Azure {REGION_MAPPINGS[region()].azure}, GCP {REGION_MAPPINGS[region()].gcp}
              </p>
            </div>

            <div class="space-y-2">
              <Label for="service">Service Type</Label>
              <Select
                id="service"
                value={serviceFilter()}
                onChange={(e) => {
                  setServiceFilter(e.currentTarget.value);
                  setResult(null);
                }}
              >
                <For each={serviceOptions()}>
                  {(option) => (
                    <option value={option.value}>{option.label}</option>
                  )}
                </For>
              </Select>
            </div>

            <Button class="w-full" onClick={findCheapest} disabled={isSearching()}>
              {isSearching() ? "Searching..." : "Find Cheapest Provider"}
            </Button>
          </CardContent>
        </Card>

        {/* Results */}
        <Show when={result()}>
          {(res) => (
            <div class="space-y-4">
              {/* Winner Card */}
              <Card class="border-green-500 bg-green-50 dark:bg-green-950">
                <CardHeader>
                  <div class="flex items-center justify-between">
                    <CardTitle class="flex items-center gap-2">
                      <svg class="h-6 w-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                      Cheapest Option
                    </CardTitle>
                    <Badge variant="success">
                      Save {res().savingsPercent.toFixed(0)}%
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <ProviderBadge provider={res().winner} />
                      <span class="text-lg font-semibold">{getProviderName(res().winner)}</span>
                    </div>
                    <Show when={res().prices[0]?.entry}>
                      <div class="text-right">
                        <div class="text-2xl font-bold text-green-600">
                          {formatMoneyPerHour(res().prices[0].entry!.unit_price)}
                        </div>
                        <div class="text-sm text-muted-foreground">
                          {res().prices[0].entry!.instance_type || res().prices[0].entry!.service_name}
                        </div>
                      </div>
                    </Show>
                  </div>
                  <Show when={res().savings > 0}>
                    <p class="mt-4 text-sm text-green-700 dark:text-green-300">
                      Saves {formatMoney({ amount: res().savings, currency: "USD", decimal_places: 2 })}/hr
                      compared to the next cheapest option
                    </p>
                  </Show>
                </CardContent>
              </Card>

              {/* All Providers Comparison */}
              <Card>
                <CardHeader>
                  <CardTitle>All Providers</CardTitle>
                  <CardDescription>
                    Price comparison for {REGION_MAPPINGS[region()].label}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div class="space-y-3">
                    <For each={res().prices}>
                      {(item, index) => (
                        <div
                          class={`flex items-center justify-between p-3 rounded-lg border ${
                            index() === 0
                              ? "border-green-500 bg-green-50 dark:bg-green-950"
                              : "border-border"
                          }`}
                        >
                          <div class="flex items-center gap-3">
                            <span class="text-lg font-bold text-muted-foreground w-6">
                              #{index() + 1}
                            </span>
                            <ProviderBadge provider={item.provider} />
                            <div>
                              <div class="font-medium">{getProviderName(item.provider)}</div>
                              <div class="text-xs text-muted-foreground">{item.region}</div>
                            </div>
                          </div>
                          <Show
                            when={item.entry}
                            fallback={
                              <span class="text-muted-foreground">No data</span>
                            }
                          >
                            <div class="text-right">
                              <div class="font-mono font-semibold">
                                {formatMoneyPerHour(item.entry!.unit_price)}
                              </div>
                              <div class="text-xs text-muted-foreground">
                                {item.entry!.instance_type || item.entry!.service_name}
                              </div>
                              <Show when={index() > 0 && res().prices[0]?.entry}>
                                <div class="text-xs text-red-500">
                                  +{formatMoney({
                                    amount: item.price - res().prices[0].price,
                                    currency: "USD",
                                    decimal_places: 2,
                                  })}/hr more
                                </div>
                              </Show>
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                  </div>
                </CardContent>
              </Card>

              {/* Details of winning entry */}
              <Show when={res().prices[0]?.entry}>
                {(entry) => (
                  <Card>
                    <CardHeader>
                      <CardTitle>Recommended Service Details</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <dl class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <dt class="text-muted-foreground">Service</dt>
                          <dd class="font-medium">{entry().service_name}</dd>
                        </div>
                        <div>
                          <dt class="text-muted-foreground">SKU/Instance</dt>
                          <dd class="font-mono">{entry().instance_type || entry().sku}</dd>
                        </div>
                        <div>
                          <dt class="text-muted-foreground">Region</dt>
                          <dd class="font-medium">{entry().region}</dd>
                        </div>
                        <div>
                          <dt class="text-muted-foreground">Pricing Unit</dt>
                          <dd class="font-medium">{entry().pricing_unit}</dd>
                        </div>
                        <div class="col-span-2">
                          <dt class="text-muted-foreground">Description</dt>
                          <dd class="font-medium">{entry().description}</dd>
                        </div>
                      </dl>
                    </CardContent>
                  </Card>
                )}
              </Show>
            </div>
          )}
        </Show>

        <Show when={!result() && !isSearching()}>
          <Card class="flex items-center justify-center min-h-[300px]">
            <CardContent class="text-center text-muted-foreground">
              <svg class="h-12 w-12 mx-auto mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <p>Select your requirements and click search to find the cheapest provider</p>
            </CardContent>
          </Card>
        </Show>
      </div>
    </div>
  );
};

export default PriceFinder;

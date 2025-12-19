import { Component, createSignal, createMemo, For, Show } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Select } from "~/components/ui/select";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Badge } from "~/components/ui/badge";
import { ProviderBadge } from "~/components/provider-badge";
import { fetchPrices, fetchPriceStats, MOCK_PRICES } from "~/lib/api-client";
import {
  type CloudProvider,
  type ResourceType,
  type PriceEntry,
  formatMoney,
  formatMoneyPerHour,
  getProviderName,
} from "~/types";

const Dashboard: Component = () => {
  const [selectedProvider, setSelectedProvider] = createSignal<CloudProvider | "all">("all");
  const [selectedResourceType, setSelectedResourceType] = createSignal<ResourceType | "all">("all");
  const [searchQuery, setSearchQuery] = createSignal("");

  // Fetch stats from API for accurate counts across all data
  const statsQuery = createQuery(() => ({
    queryKey: ["price-stats"],
    queryFn: fetchPriceStats,
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: 1,
  }));

  // Fetch prices for table display (limited for performance)
  const pricesQuery = createQuery(() => ({
    queryKey: ["prices", selectedProvider(), selectedResourceType()],
    queryFn: () => fetchPrices({
      limit: 100,
      provider: selectedProvider() === "all" ? undefined : selectedProvider(),
      resource_type: selectedResourceType() === "all" ? undefined : selectedResourceType(),
    }),
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: 1,
    placeholderData: MOCK_PRICES,
  }));

  const prices = () => pricesQuery.data ?? MOCK_PRICES;

  const filteredPrices = createMemo(() => {
    return prices().filter((price) => {
      if (searchQuery()) {
        const query = searchQuery().toLowerCase();
        return (
          price.service_name.toLowerCase().includes(query) ||
          price.instance_type?.toLowerCase().includes(query) ||
          price.description.toLowerCase().includes(query)
        );
      }
      return true;
    });
  });

  // Use stats API for accurate totals, fall back to counting fetched data
  const summaryStats = createMemo(() => {
    const stats = statsQuery.data;
    if (stats) {
      return {
        total: stats.total_count,
        byProvider: stats.by_provider,
      };
    }
    // Fallback to counting fetched data
    const all = prices();
    return {
      total: all.length,
      byProvider: {
        aws: all.filter((p) => p.provider === "aws").length,
        azure: all.filter((p) => p.provider === "azure").length,
        gcp: all.filter((p) => p.provider === "gcp").length,
      },
    };
  });

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Pricing Dashboard</h1>
        <p class="text-muted-foreground mt-2">
          Compare cloud pricing across AWS, Azure, and GCP
        </p>
      </div>

      {/* Summary Cards */}
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Total Prices</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{summaryStats().total}</div>
            <p class="text-xs text-muted-foreground">Across all providers</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">AWS</CardTitle>
            <Badge variant="aws">AWS</Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{summaryStats().byProvider.aws}</div>
            <p class="text-xs text-muted-foreground">Price entries</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Azure</CardTitle>
            <Badge variant="azure">Azure</Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{summaryStats().byProvider.azure}</div>
            <p class="text-xs text-muted-foreground">Price entries</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">GCP</CardTitle>
            <Badge variant="gcp">GCP</Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{summaryStats().byProvider.gcp}</div>
            <p class="text-xs text-muted-foreground">Price entries</p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Filters</CardTitle>
          <CardDescription>Filter prices by provider, type, or search</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="grid gap-4 md:grid-cols-3">
            <div class="space-y-2">
              <Label for="provider">Provider</Label>
              <Select
                id="provider"
                value={selectedProvider()}
                onChange={(e) => setSelectedProvider(e.currentTarget.value as CloudProvider | "all")}
              >
                <option value="all">All Providers</option>
                <option value="aws">AWS</option>
                <option value="azure">Azure</option>
                <option value="gcp">GCP</option>
              </Select>
            </div>
            <div class="space-y-2">
              <Label for="resource-type">Resource Type</Label>
              <Select
                id="resource-type"
                value={selectedResourceType()}
                onChange={(e) => setSelectedResourceType(e.currentTarget.value as ResourceType | "all")}
              >
                <option value="all">All Types</option>
                <option value="compute">Compute</option>
                <option value="storage">Storage</option>
                <option value="database">Database</option>
                <option value="kubernetes">Kubernetes</option>
                <option value="network">Network</option>
              </Select>
            </div>
            <div class="space-y-2">
              <Label for="search">Search</Label>
              <Input
                id="search"
                placeholder="Search by name, instance type..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Price Table */}
      <Card>
        <CardHeader>
          <CardTitle>Prices</CardTitle>
          <CardDescription>
            Showing {filteredPrices().length} of {summaryStats().total.toLocaleString()} total prices
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b">
                  <th class="text-left py-3 px-4 font-medium">Provider</th>
                  <th class="text-left py-3 px-4 font-medium">Service</th>
                  <th class="text-left py-3 px-4 font-medium">Instance Type</th>
                  <th class="text-left py-3 px-4 font-medium">Region</th>
                  <th class="text-left py-3 px-4 font-medium">Type</th>
                  <th class="text-right py-3 px-4 font-medium">Price</th>
                </tr>
              </thead>
              <tbody>
                <For each={filteredPrices()}>
                  {(price) => (
                    <tr class="border-b hover:bg-muted/50">
                      <td class="py-3 px-4">
                        <ProviderBadge provider={price.provider} />
                      </td>
                      <td class="py-3 px-4">
                        <div class="font-medium">{price.service_name}</div>
                        <div class="text-xs text-muted-foreground">{price.description}</div>
                      </td>
                      <td class="py-3 px-4">
                        <code class="text-xs bg-muted px-1 py-0.5 rounded">
                          {price.instance_type || "N/A"}
                        </code>
                      </td>
                      <td class="py-3 px-4 text-muted-foreground">{price.region}</td>
                      <td class="py-3 px-4">
                        <Badge variant="outline">{price.resource_type}</Badge>
                      </td>
                      <td class="py-3 px-4 text-right font-mono">
                        {formatMoneyPerHour(price.unit_price)}
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
          <Show when={filteredPrices().length === 0}>
            <div class="text-center py-8 text-muted-foreground">
              No prices match your filters
            </div>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default Dashboard;

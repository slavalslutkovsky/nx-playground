import { Component, For, Show } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { Link } from "@tanstack/solid-router";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Badge } from "~/components/ui/badge";
import { ProviderBadge } from "~/components/provider-badge";
import { fetchCncfTools, MOCK_CNCF_TOOLS } from "~/lib/api-client";
import type { CncfTool, CncfMaturity, CncfToolCategory } from "~/types";

function getMaturityColor(maturity: CncfMaturity): "default" | "secondary" | "success" {
  switch (maturity) {
    case "graduated":
      return "success";
    case "incubating":
      return "secondary";
    case "sandbox":
      return "default";
  }
}

function getCategoryIcon(category: CncfToolCategory): string {
  switch (category) {
    case "database":
      return "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4";
    case "cache":
      return "M13 10V3L4 14h7v7l9-11h-7z";
    case "message_queue":
      return "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z";
    case "storage":
      return "M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4";
    case "observability":
      return "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z";
    default:
      return "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4";
  }
}

const CncfTools: Component = () => {
  // Fetch CNCF tools from API (falls back to mock if API unavailable)
  const toolsQuery = createQuery(() => ({
    queryKey: ["cncf-tools"],
    queryFn: fetchCncfTools,
    staleTime: 1000 * 60 * 10, // 10 minutes (static data changes rarely)
    retry: 1,
    placeholderData: MOCK_CNCF_TOOLS,
  }));

  const tools = () => toolsQuery.data ?? MOCK_CNCF_TOOLS;

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">CNCF Tools</h1>
        <p class="text-muted-foreground mt-2">
          Self-managed Kubernetes operators that can replace managed cloud services
        </p>
      </div>

      {/* Info Banner */}
      <Card class="bg-primary/5 border-primary/20">
        <CardContent class="pt-6">
          <div class="flex items-start gap-4">
            <svg
              class="h-6 w-6 text-primary flex-shrink-0 mt-0.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div>
              <h3 class="font-semibold">Why Self-Managed?</h3>
              <p class="text-sm text-muted-foreground mt-1">
                CNCF tools running on Kubernetes can save 30-60% vs managed services at scale.
                They offer portability across clouds and full control over configuration.
                The trade-off is operational overhead - best suited for teams with dedicated
                platform engineering.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tools Grid */}
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <For each={tools()}>
          {(tool) => (
            <Card class="flex flex-col">
              <CardHeader>
                <div class="flex items-start justify-between">
                  <div class="flex items-center gap-3">
                    <div class="p-2 bg-primary/10 rounded-lg">
                      <svg
                        class="h-6 w-6 text-primary"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d={getCategoryIcon(tool.category)}
                        />
                      </svg>
                    </div>
                    <div>
                      <CardTitle class="text-lg">{tool.name}</CardTitle>
                      <CardDescription class="capitalize">{tool.category.replace("_", " ")}</CardDescription>
                    </div>
                  </div>
                  <Badge variant={getMaturityColor(tool.maturity)}>{tool.maturity}</Badge>
                </div>
              </CardHeader>
              <CardContent class="flex-1 space-y-4">
                <p class="text-sm text-muted-foreground">{tool.description}</p>

                {/* GitHub Stars */}
                <Show when={tool.github_stars}>
                  <div class="flex items-center gap-1 text-sm text-muted-foreground">
                    <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                    </svg>
                    <span>{tool.github_stars?.toLocaleString()} stars</span>
                  </div>
                </Show>

                {/* Resource Requirements */}
                <div class="space-y-2">
                  <h4 class="text-sm font-medium">HA Requirements</h4>
                  <div class="grid grid-cols-3 gap-2 text-xs">
                    <div class="bg-muted p-2 rounded text-center">
                      <div class="font-semibold">
                        {(tool.ha_requirements.cpu_millicores / 1000).toFixed(1)}
                      </div>
                      <div class="text-muted-foreground">vCPU</div>
                    </div>
                    <div class="bg-muted p-2 rounded text-center">
                      <div class="font-semibold">
                        {(tool.ha_requirements.memory_mb / 1024).toFixed(1)}
                      </div>
                      <div class="text-muted-foreground">GB RAM</div>
                    </div>
                    <div class="bg-muted p-2 rounded text-center">
                      <div class="font-semibold">{tool.ha_requirements.replicas}x</div>
                      <div class="text-muted-foreground">Replicas</div>
                    </div>
                  </div>
                </div>

                {/* Managed Equivalents */}
                <div class="space-y-2">
                  <h4 class="text-sm font-medium">Replaces</h4>
                  <div class="flex flex-wrap gap-2">
                    <For each={tool.managed_equivalents}>
                      {(equiv) => (
                        <div class="flex items-center gap-1">
                          <ProviderBadge provider={equiv.provider} />
                        </div>
                      )}
                    </For>
                  </div>
                </div>

                {/* Features */}
                <div class="space-y-2">
                  <h4 class="text-sm font-medium">Key Features</h4>
                  <ul class="text-xs text-muted-foreground space-y-1">
                    <For each={tool.included_features.slice(0, 4)}>
                      {(feature) => (
                        <li class="flex items-center gap-1">
                          <svg
                            class="h-3 w-3 text-green-500"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                          >
                            <path
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              stroke-width="2"
                              d="M5 13l4 4L19 7"
                            />
                          </svg>
                          {feature}
                        </li>
                      )}
                    </For>
                    <Show when={tool.included_features.length > 4}>
                      <li class="text-muted-foreground">
                        +{tool.included_features.length - 4} more features
                      </li>
                    </Show>
                  </ul>
                </div>
              </CardContent>
              <div class="p-6 pt-0 mt-auto flex gap-2">
                <a href={tool.project_url} target="_blank" rel="noopener noreferrer" class="flex-1">
                  <Button variant="outline" class="w-full">
                    Docs
                  </Button>
                </a>
                <Link to={`/tco?tool=${tool.id}`} class="flex-1">
                  <Button class="w-full">Calculate TCO</Button>
                </Link>
              </div>
            </Card>
          )}
        </For>
      </div>

      {/* Ops Hours Comparison */}
      <Card>
        <CardHeader>
          <CardTitle>Operational Overhead Comparison</CardTitle>
          <CardDescription>Estimated monthly hours required for each tool</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b">
                  <th class="text-left py-3 px-4 font-medium">Tool</th>
                  <th class="text-right py-3 px-4 font-medium">Setup (one-time)</th>
                  <th class="text-right py-3 px-4 font-medium">Minimal</th>
                  <th class="text-right py-3 px-4 font-medium">HA</th>
                  <th class="text-right py-3 px-4 font-medium">Production</th>
                </tr>
              </thead>
              <tbody>
                <For each={tools()}>
                  {(tool) => (
                    <tr class="border-b hover:bg-muted/50">
                      <td class="py-3 px-4 font-medium">{tool.name}</td>
                      <td class="py-3 px-4 text-right">{tool.ops_hours.initial_setup_hours}h</td>
                      <td class="py-3 px-4 text-right">{tool.ops_hours.minimal_monthly_hours}h/mo</td>
                      <td class="py-3 px-4 text-right">{tool.ops_hours.ha_monthly_hours}h/mo</td>
                      <td class="py-3 px-4 text-right">{tool.ops_hours.production_monthly_hours}h/mo</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default CncfTools;

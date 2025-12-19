import { Component, For, Show, createSignal } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Badge } from "~/components/ui/badge";
import { fetchCncfLandscape, fetchCncfRecommendations } from "~/lib/api-client";
import type { CncfToolCategory, CncfMaturity, CncfToolEnriched, ToolRecommendation } from "~/types";

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

function getCategoryLabel(category: CncfToolCategory): string {
  switch (category) {
    case "database":
      return "Database";
    case "cache":
      return "Cache";
    case "message_queue":
      return "Message Queue";
    case "storage":
      return "Storage";
    case "observability":
      return "Observability";
    case "service_mesh":
      return "Service Mesh";
    case "gitops":
      return "GitOps";
    default:
      return category;
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
    case "service_mesh":
      return "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4";
    case "gitops":
      return "M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2";
    default:
      return "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4";
  }
}

const CATEGORIES: CncfToolCategory[] = [
  "database",
  "observability",
  "message_queue",
  "storage",
  "service_mesh",
  "gitops",
];

const CncfLandscape: Component = () => {
  const [selectedCategory, setSelectedCategory] = createSignal<CncfToolCategory>("database");

  // Fetch all CNCF landscape data
  const landscapeQuery = createQuery(() => ({
    queryKey: ["cncf-landscape"],
    queryFn: fetchCncfLandscape,
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: 2,
  }));

  // Fetch recommendations for selected category
  const recommendationsQuery = createQuery(() => ({
    queryKey: ["cncf-recommendations", selectedCategory()],
    queryFn: () => fetchCncfRecommendations(selectedCategory()),
    staleTime: 1000 * 60 * 5,
    retry: 2,
  }));

  const currentCategory = () => {
    return landscapeQuery.data?.categories.find(c => c.category === selectedCategory());
  };

  const getRecommendation = (toolId: string): ToolRecommendation | undefined => {
    return recommendationsQuery.data?.recommendations.find(r => r.tool_id === toolId);
  };

  return (
    <div class="space-y-8">
      {/* Header */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">CNCF Landscape</h1>
        <p class="text-muted-foreground mt-2">
          Real-time data from the CNCF landscape with AI-powered recommendations
        </p>
        <Show when={landscapeQuery.data}>
          <p class="text-sm text-muted-foreground mt-1">
            {landscapeQuery.data!.total_tools} tools across {landscapeQuery.data!.categories.length} categories
            {" | "}Last updated: {new Date(landscapeQuery.data!.last_updated).toLocaleDateString()}
          </p>
        </Show>
      </div>

      {/* Category Tabs */}
      <div class="flex flex-wrap gap-2">
        <For each={CATEGORIES}>
          {(category) => (
            <Button
              variant={selectedCategory() === category ? "default" : "outline"}
              onClick={() => setSelectedCategory(category)}
              class="flex items-center gap-2"
            >
              <svg
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d={getCategoryIcon(category)}
                />
              </svg>
              {getCategoryLabel(category)}
            </Button>
          )}
        </For>
      </div>

      {/* Top Pick */}
      <Show when={recommendationsQuery.data?.top_pick}>
        <Card class="bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800">
          <CardContent class="pt-6">
            <div class="flex items-start gap-4">
              <div class="p-2 bg-green-500 rounded-lg">
                <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <div>
                <h3 class="font-semibold text-green-900 dark:text-green-100">
                  Top Pick: {recommendationsQuery.data!.recommendations.find(r => r.tool_id === recommendationsQuery.data!.top_pick)?.tool_name}
                </h3>
                <p class="text-sm text-green-700 dark:text-green-300 mt-1">
                  {recommendationsQuery.data!.top_pick_reason}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </Show>

      {/* Loading State */}
      <Show when={landscapeQuery.isLoading || recommendationsQuery.isLoading}>
        <div class="flex items-center justify-center py-12">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>
      </Show>

      {/* Error State */}
      <Show when={landscapeQuery.error}>
        <Card class="bg-red-50 dark:bg-red-950 border-red-200">
          <CardContent class="pt-6">
            <p class="text-red-700 dark:text-red-300">
              Failed to load CNCF landscape: {(landscapeQuery.error as Error).message}
            </p>
          </CardContent>
        </Card>
      </Show>

      {/* Tools Grid */}
      <Show when={currentCategory()}>
        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          <For each={currentCategory()!.tools}>
            {(tool) => {
              const recommendation = () => getRecommendation(tool.id);
              return (
                <ToolCard tool={tool} recommendation={recommendation()} />
              );
            }}
          </For>
        </div>
      </Show>

      {/* Empty State */}
      <Show when={currentCategory()?.tools.length === 0}>
        <Card>
          <CardContent class="pt-6 text-center">
            <p class="text-muted-foreground">No tools found in this category</p>
          </CardContent>
        </Card>
      </Show>
    </div>
  );
};

interface ToolCardProps {
  tool: CncfToolEnriched;
  recommendation?: ToolRecommendation;
}

const ToolCard: Component<ToolCardProps> = (props) => {
  const [showDetails, setShowDetails] = createSignal(false);

  return (
    <Card class="flex flex-col">
      <CardHeader>
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-3">
            <Show when={props.tool.logo_url} fallback={
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
                    d={getCategoryIcon(props.tool.category)}
                  />
                </svg>
              </div>
            }>
              <img
                src={props.tool.logo_url!}
                alt={props.tool.name}
                class="h-10 w-10 rounded-lg object-contain"
              />
            </Show>
            <div>
              <CardTitle class="text-lg">{props.tool.name}</CardTitle>
              <CardDescription>{getCategoryLabel(props.tool.category)}</CardDescription>
            </div>
          </div>
          <div class="flex flex-col items-end gap-1">
            <Badge variant={getMaturityColor(props.tool.maturity)}>{props.tool.maturity}</Badge>
            <Show when={props.recommendation}>
              <Badge variant="outline" class="text-xs">
                Score: {props.recommendation!.score}
              </Badge>
            </Show>
          </div>
        </div>
      </CardHeader>
      <CardContent class="flex-1 space-y-4">
        <p class="text-sm text-muted-foreground line-clamp-2">{props.tool.description}</p>

        {/* GitHub Stats */}
        <Show when={props.tool.github_stats}>
          <div class="flex flex-wrap gap-3 text-sm text-muted-foreground">
            <div class="flex items-center gap-1">
              <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
              </svg>
              <span>{props.tool.github_stats!.stars.toLocaleString()}</span>
            </div>
            <div class="flex items-center gap-1">
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
              </svg>
              <span>{props.tool.github_stats!.forks.toLocaleString()}</span>
            </div>
            <div class="flex items-center gap-1">
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
              </svg>
              <span>{props.tool.github_stats!.contributors}</span>
            </div>
          </div>
        </Show>

        {/* Pros/Cons from AI Recommendation */}
        <Show when={props.recommendation}>
          <div class="space-y-3">
            {/* Pros */}
            <div>
              <h4 class="text-sm font-medium text-green-700 dark:text-green-400 mb-1">Pros</h4>
              <ul class="text-xs space-y-1">
                <For each={props.recommendation!.pros.slice(0, showDetails() ? undefined : 3)}>
                  {(pro) => (
                    <li class="flex items-start gap-1 text-muted-foreground">
                      <svg class="h-3 w-3 text-green-500 mt-0.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                      <span>{pro}</span>
                    </li>
                  )}
                </For>
              </ul>
            </div>

            {/* Cons */}
            <div>
              <h4 class="text-sm font-medium text-red-700 dark:text-red-400 mb-1">Cons</h4>
              <ul class="text-xs space-y-1">
                <For each={props.recommendation!.cons.slice(0, showDetails() ? undefined : 3)}>
                  {(con) => (
                    <li class="flex items-start gap-1 text-muted-foreground">
                      <svg class="h-3 w-3 text-red-500 mt-0.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                      <span>{con}</span>
                    </li>
                  )}
                </For>
              </ul>
            </div>

            {/* Show More Button */}
            <Show when={props.recommendation!.pros.length > 3 || props.recommendation!.cons.length > 3}>
              <Button
                variant="ghost"
                size="sm"
                class="text-xs"
                onClick={() => setShowDetails(!showDetails())}
              >
                {showDetails() ? "Show Less" : "Show More"}
              </Button>
            </Show>

            {/* Best For / Avoid If */}
            <Show when={showDetails()}>
              <div class="grid grid-cols-2 gap-3 pt-2 border-t">
                <div>
                  <h4 class="text-xs font-medium text-blue-700 dark:text-blue-400 mb-1">Best For</h4>
                  <ul class="text-xs space-y-1">
                    <For each={props.recommendation!.best_for}>
                      {(item) => (
                        <li class="text-muted-foreground">{item}</li>
                      )}
                    </For>
                  </ul>
                </div>
                <div>
                  <h4 class="text-xs font-medium text-orange-700 dark:text-orange-400 mb-1">Avoid If</h4>
                  <ul class="text-xs space-y-1">
                    <For each={props.recommendation!.avoid_if}>
                      {(item) => (
                        <li class="text-muted-foreground">{item}</li>
                      )}
                    </For>
                  </ul>
                </div>
              </div>
            </Show>
          </div>
        </Show>
      </CardContent>
      <div class="p-6 pt-0 mt-auto flex gap-2">
        <a href={props.tool.project_url} target="_blank" rel="noopener noreferrer" class="flex-1">
          <Button variant="outline" class="w-full">
            Project
          </Button>
        </a>
        <Show when={props.tool.repo_url}>
          <a href={props.tool.repo_url!} target="_blank" rel="noopener noreferrer" class="flex-1">
            <Button variant="outline" class="w-full">
              GitHub
            </Button>
          </a>
        </Show>
      </div>
    </Card>
  );
};

export default CncfLandscape;

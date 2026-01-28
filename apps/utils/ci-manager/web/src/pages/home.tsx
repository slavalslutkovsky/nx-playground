import { createMutation, createQuery } from '@tanstack/solid-query';
import type { Component } from 'solid-js';
import { createSignal, createUniqueId, Show } from 'solid-js';
import { LanguageSelector } from '../components/language-selector';
import { ProviderSelector } from '../components/provider-selector';
import { Button } from '../components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '../components/ui/card';
import { Checkbox } from '../components/ui/checkbox';
import {
  fetchLanguages,
  fetchProviders,
  generatePipeline,
} from '../lib/api-client';

export const HomePage: Component = () => {
  const lintId = createUniqueId();
  const testId = createUniqueId();
  const buildId = createUniqueId();
  const cacheId = createUniqueId();

  const [selectedLanguages, setSelectedLanguages] = createSignal<string[]>([]);
  const [selectedProvider, setSelectedProvider] = createSignal<string | null>(
    null,
  );
  const [options, setOptions] = createSignal({
    lint: true,
    test: true,
    build: true,
    cache: true,
  });

  const languagesQuery = createQuery(() => ({
    queryKey: ['languages'],
    queryFn: fetchLanguages,
  }));

  const providersQuery = createQuery(() => ({
    queryKey: ['providers'],
    queryFn: fetchProviders,
  }));

  const generateMutation = createMutation(() => ({
    mutationFn: generatePipeline,
  }));

  const toggleLanguage = (languageId: string) => {
    setSelectedLanguages((prev) =>
      prev.includes(languageId)
        ? prev.filter((id) => id !== languageId)
        : [...prev, languageId],
    );
  };

  const toggleOption = (key: keyof typeof options) => {
    setOptions((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const handleGenerate = () => {
    const provider = selectedProvider();
    if (selectedLanguages().length === 0 || !provider) {
      return;
    }

    generateMutation.mutate({
      languages: selectedLanguages(),
      provider,
      options: options(),
    });
  };

  const canGenerate = () =>
    selectedLanguages().length > 0 && selectedProvider() !== null;

  return (
    <div class="min-h-screen bg-gray-50 py-8">
      <div class="max-w-4xl mx-auto px-4 space-y-8">
        <div class="text-center">
          <h1 class="text-3xl font-bold text-gray-900">CI Pipeline Manager</h1>
          <p class="text-gray-600 mt-2">
            Configure your CI pipeline by selecting languages and a CI provider
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>1. Select Languages</CardTitle>
            <CardDescription>
              Choose the languages and runtimes used in your project
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Show
              when={!languagesQuery.isLoading}
              fallback={
                <div class="text-center py-4">Loading languages...</div>
              }
            >
              <Show
                when={languagesQuery.data}
                fallback={
                  <div class="text-red-500">Failed to load languages</div>
                }
              >
                {(languages) => (
                  <LanguageSelector
                    languages={languages()}
                    selectedLanguages={selectedLanguages()}
                    onToggle={toggleLanguage}
                  />
                )}
              </Show>
            </Show>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>2. Select CI Provider</CardTitle>
            <CardDescription>Choose your CI/CD platform</CardDescription>
          </CardHeader>
          <CardContent>
            <Show
              when={!providersQuery.isLoading}
              fallback={
                <div class="text-center py-4">Loading providers...</div>
              }
            >
              <Show
                when={providersQuery.data}
                fallback={
                  <div class="text-red-500">Failed to load providers</div>
                }
              >
                {(providers) => (
                  <ProviderSelector
                    providers={providers()}
                    selectedProvider={selectedProvider()}
                    onSelect={setSelectedProvider}
                  />
                )}
              </Show>
            </Show>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>3. Pipeline Options</CardTitle>
            <CardDescription>
              Configure what steps to include in your pipeline
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <Checkbox
                id={lintId}
                label="Lint"
                checked={options().lint}
                onChange={() => toggleOption('lint')}
              />
              <Checkbox
                id={testId}
                label="Test"
                checked={options().test}
                onChange={() => toggleOption('test')}
              />
              <Checkbox
                id={buildId}
                label="Build"
                checked={options().build}
                onChange={() => toggleOption('build')}
              />
              <Checkbox
                id={cacheId}
                label="Cache"
                checked={options().cache}
                onChange={() => toggleOption('cache')}
              />
            </div>
          </CardContent>
        </Card>

        <div class="flex justify-center">
          <Button
            size="lg"
            onClick={handleGenerate}
            disabled={!canGenerate() || generateMutation.isPending}
          >
            {generateMutation.isPending ? 'Generating...' : 'Generate Pipeline'}
          </Button>
        </div>

        <Show when={generateMutation.isError}>
          <Card class="border-red-200 bg-red-50">
            <CardContent class="p-4">
              <p class="text-red-700">
                Error: {generateMutation.error?.message}
              </p>
            </CardContent>
          </Card>
        </Show>

        <Show when={generateMutation.data}>
          {(result) => (
            <Card>
              <CardHeader>
                <CardTitle>Generated Pipeline</CardTitle>
                <CardDescription>
                  {result().provider} configuration for{' '}
                  {result().languages.join(', ')}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <span class="text-sm text-gray-500">
                      Save as:{' '}
                      <code class="bg-gray-100 px-2 py-1 rounded">
                        {result().configFile}
                      </code>
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        navigator.clipboard.writeText(result().yaml);
                      }}
                    >
                      Copy to Clipboard
                    </Button>
                  </div>
                  <pre class="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code>{result().yaml}</code>
                  </pre>
                </div>
              </CardContent>
            </Card>
          )}
        </Show>
      </div>
    </div>
  );
};

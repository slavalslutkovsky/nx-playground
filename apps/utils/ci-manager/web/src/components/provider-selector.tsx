import type { Component } from 'solid-js';
import { For } from 'solid-js';
import type { CIProvider } from '../lib/api-client';
import { cn } from '../lib/utils';
import { Card, CardContent, CardDescription } from './ui/card';

interface ProviderSelectorProps {
  providers: CIProvider[];
  selectedProvider: string | null;
  onSelect: (providerId: string) => void;
}

export const ProviderSelector: Component<ProviderSelectorProps> = (props) => {
  return (
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <For each={props.providers}>
        {(provider) => {
          const isSelected = () => props.selectedProvider === provider.id;

          return (
            <Card
              class={cn(
                'cursor-pointer transition-all hover:shadow-md',
                isSelected() && 'ring-2 ring-blue-500 bg-blue-50',
              )}
              onClick={() => props.onSelect(provider.id)}
            >
              <CardContent class="p-4 flex flex-col items-center justify-center text-center">
                <span class="text-3xl mb-2">{provider.icon}</span>
                <span class="font-medium">{provider.name}</span>
                <CardDescription class="mt-1 text-xs">
                  {provider.configFile}
                </CardDescription>
              </CardContent>
            </Card>
          );
        }}
      </For>
    </div>
  );
};

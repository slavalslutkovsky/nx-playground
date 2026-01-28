import type { Component } from 'solid-js';
import { For } from 'solid-js';
import type { Language } from '../lib/api-client';
import { cn } from '../lib/utils';
import { Card, CardContent } from './ui/card';

interface LanguageSelectorProps {
  languages: Language[];
  selectedLanguages: string[];
  onToggle: (languageId: string) => void;
}

export const LanguageSelector: Component<LanguageSelectorProps> = (props) => {
  return (
    <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
      <For each={props.languages}>
        {(language) => {
          const isSelected = () =>
            props.selectedLanguages.includes(language.id);

          return (
            <Card
              class={cn(
                'cursor-pointer transition-all hover:shadow-md',
                isSelected() && 'ring-2 ring-blue-500 bg-blue-50',
              )}
              onClick={() => props.onToggle(language.id)}
            >
              <CardContent class="p-4 flex flex-col items-center justify-center text-center">
                <span class="text-3xl mb-2">{language.icon}</span>
                <span class="font-medium">{language.name}</span>
                <span class="text-xs text-gray-500 mt-1">
                  {language.buildTools[0]}
                </span>
              </CardContent>
            </Card>
          );
        }}
      </For>
    </div>
  );
};

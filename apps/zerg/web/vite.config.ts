/// <reference types="vitest" />
/// <reference types="vite/client" />

import path from 'node:path';
import tailwindcss from '@tailwindcss/vite';
import devtools from 'solid-devtools/vite';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [devtools(), solidPlugin(), tailwindcss()],
  // base: '/web/',
  server: {
    port: 3000,
  },
  build: {
    target: 'esnext',
  },
  resolve: {
    conditions: ['development', 'browser'],
    alias: {
      '@domain/tasks': path.resolve(
        __dirname,
        '../../../libs/domains/tasks/types/index.ts',
      ),
    },
  },
});

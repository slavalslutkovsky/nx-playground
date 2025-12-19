/// <reference types="vitest" />
/// <reference types="vite/client" />

import path from 'node:path';
import tailwindcss from '@tailwindcss/vite';
import devtools from 'solid-devtools/vite';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [devtools(), solidPlugin(), tailwindcss()],
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
      '@nx-playground/auth-solid': path.resolve(
        __dirname,
        '../../../libs/web/auth-solid/src',
      ),
    },
    dedupe: ['solid-js', '@tanstack/solid-router', '@tanstack/solid-query'],
  },
  optimizeDeps: {
    include: ['solid-js', '@tanstack/solid-router', '@tanstack/solid-query'],
  },
});

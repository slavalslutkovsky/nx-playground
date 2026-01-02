/// <reference types="vitest" />
/// <reference types="vite/client" />

import path from 'node:path';
import { sentryVitePlugin } from '@sentry/vite-plugin';
import tailwindcss from '@tailwindcss/vite';
import devtools from 'solid-devtools/vite';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig(({ mode }) => ({
  plugins: [
    devtools(),
    solidPlugin(),
    tailwindcss(),
    // Sentry plugin for source map uploads (production only)
    mode === 'production' &&
      process.env.SENTRY_AUTH_TOKEN &&
      sentryVitePlugin({
        org: process.env.SENTRY_ORG,
        project: process.env.SENTRY_PROJECT,
        authToken: process.env.SENTRY_AUTH_TOKEN,
        sourcemaps: {
          filesToDeleteAfterUpload: ['**/*.map'],
        },
        telemetry: false,
      }),
  ].filter(Boolean),
  server: {
    port: 3000,
  },
  build: {
    target: 'esnext',
    sourcemap: true, // Required for Sentry source maps
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
}));

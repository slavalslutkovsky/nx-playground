import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import devtools from "solid-devtools/vite";
import path from "path";

export default defineConfig({
  plugins: [
    devtools({
      autoname: true,
    }),
    solid(),
    tailwindcss(),
  ],
  server: {
    port: 3001,
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
    },
  },
  build: {
    target: "esnext",
  },
  resolve: {
    alias: {
      "~": "/src",
      "@nx-playground/auth-solid": path.resolve(__dirname, "../../../libs/web/auth-solid/src"),
    },
    dedupe: ["solid-js", "@tanstack/solid-router", "@tanstack/solid-query"],
  },
  optimizeDeps: {
    include: ["solid-js", "@tanstack/solid-router", "@tanstack/solid-query"],
  },
});

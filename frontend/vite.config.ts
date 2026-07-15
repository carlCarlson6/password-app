/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import process from "node:process";
import { defineConfig } from "vite";

// Dev-only: forward API calls to the Axum backend. The default targets a
// native `cargo run -p api`; docker-compose overrides it so the proxy reaches
// the backend container by service name instead.
const apiProxyTarget = process.env.API_PROXY_TARGET ?? "http://127.0.0.1:8080";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      "/api": apiProxyTarget,
    },
  },
  test: {
    environment: "node",
  },
});

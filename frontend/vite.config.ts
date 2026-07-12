/// <reference types="vitest/config" />
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Dev-only: forward API calls to the Axum backend (`cargo run -p api`).
      "/api": "http://127.0.0.1:8080",
    },
  },
  test: {
    environment: "node",
  },
});

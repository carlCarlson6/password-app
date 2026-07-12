// Tests for src/domain/health.ts — same convention as the backend:
// tests live under tests/, mirroring the src/ folder structure.

import { describe, expect, it } from "vitest";

import { isHealthy } from "../../src/domain/health";

describe("isHealthy", () => {
  it("is true only when status is ok", () => {
    expect(isHealthy({ status: "ok", database: "up" })).toBe(true);
    expect(isHealthy({ status: "degraded", database: "down" })).toBe(false);
  });
});

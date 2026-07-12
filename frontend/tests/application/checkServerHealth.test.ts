// Tests for src/application/checkServerHealth.ts.

import { describe, expect, it } from "vitest";

import { makeCheckServerHealth } from "../../src/application/checkServerHealth";
import type { HealthGateway } from "../../src/application/ports";

describe("checkServerHealth", () => {
  it("returns the gateway's report untouched", async () => {
    const gateway: HealthGateway = {
      fetchHealth: async () => ({ status: "ok", database: "up" }),
    };

    await expect(makeCheckServerHealth(gateway)()).resolves.toEqual({
      status: "ok",
      database: "up",
    });
  });

  it("degrades instead of throwing when the gateway is unreachable", async () => {
    const gateway: HealthGateway = {
      fetchHealth: async () => {
        throw new Error("network down");
      },
    };

    await expect(makeCheckServerHealth(gateway)()).resolves.toEqual({
      status: "degraded",
      database: "down",
    });
  });
});

import type { HealthGateway } from "../../application/ports";
import type { ServerHealth } from "../../domain/health";

/**
 * Fetch adapter for the HealthGateway port — the only place that knows the
 * wire path. (UI components never call fetch directly.)
 */
export function makeHttpHealthGateway(baseUrl = ""): HealthGateway {
  return {
    async fetchHealth(): Promise<ServerHealth> {
      const response = await fetch(`${baseUrl}/api/health`);
      // 503 still carries a valid health body ("degraded"); anything else
      // unexpected is a transport failure.
      if (!response.ok && response.status !== 503) {
        throw new Error(`health request failed: HTTP ${response.status}`);
      }
      return (await response.json()) as ServerHealth;
    },
  };
}

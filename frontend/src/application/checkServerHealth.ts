import type { ServerHealth } from "../domain/health";
import type { HealthGateway } from "./ports";

/**
 * Walking-skeleton use case: report backend reachability.
 * A gateway failure is a health answer, not an exception to leak to the UI.
 */
export function makeCheckServerHealth(gateway: HealthGateway) {
  return async function checkServerHealth(): Promise<ServerHealth> {
    try {
      return await gateway.fetchHealth();
    } catch {
      return { status: "degraded", database: "down" };
    }
  };
}

export type CheckServerHealth = ReturnType<typeof makeCheckServerHealth>;

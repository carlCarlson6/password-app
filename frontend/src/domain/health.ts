// Client domain layer: pure types and logic — no fetch, no WebCrypto, no React.

/** Health of the backend as reported by the walking-skeleton endpoint. */
export type ComponentStatus = "up" | "down";

export interface ServerHealth {
  status: "ok" | "degraded";
  database: ComponentStatus;
}

export function isHealthy(health: ServerHealth): boolean {
  return health.status === "ok";
}

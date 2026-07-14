import type { ServerHealth } from "../../domain/health";

/**
 * Driven port: how the application layer reaches the backend.
 * Implemented in `infrastructure/` (fetch); stubbed in tests.
 */
export interface HealthGateway {
  fetchHealth(): Promise<ServerHealth>;
}

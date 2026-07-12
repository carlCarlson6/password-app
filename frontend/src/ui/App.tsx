import { useEffect, useState } from "react";

import type { CheckServerHealth } from "../application/checkServerHealth";
import { type ServerHealth, isHealthy } from "../domain/health";

interface AppProps {
  // The use case arrives as a prop from the composition root (main.tsx):
  // the UI layer never builds gateways or touches fetch itself.
  checkServerHealth: CheckServerHealth;
}

export function App({ checkServerHealth }: AppProps) {
  const [health, setHealth] = useState<ServerHealth | null>(null);

  useEffect(() => {
    let cancelled = false;
    void checkServerHealth().then((result) => {
      if (!cancelled) setHealth(result);
    });
    return () => {
      cancelled = true;
    };
  }, [checkServerHealth]);

  return (
    <main>
      <h1>Password App</h1>
      <p>Walking skeleton: UI → Axum → use case → SQLite and back.</p>
      {health === null ? (
        <p>Checking server…</p>
      ) : (
        <p role="status">
          Server is {isHealthy(health) ? "healthy" : "degraded"} — database{" "}
          {health.database}.
        </p>
      )}
    </main>
  );
}

// Composition root: the only place adapters and use cases are wired together.

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { makeCheckServerHealth } from "./application/checkServerHealth";
import { makeHttpHealthGateway } from "./infrastructure/http/healthGateway";
import { App } from "./ui/App";

const checkServerHealth = makeCheckServerHealth(makeHttpHealthGateway());

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App checkServerHealth={checkServerHealth} />
  </StrictMode>,
);

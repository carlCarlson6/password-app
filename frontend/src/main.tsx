// Composition root: the only place adapters and use cases are wired together.

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

// Global stylesheet (Tailwind + the retro terminal theme). Imported once at
// the composition root; components only use the class names it defines.
import "./ui/index.css";

import { makeCheckServerHealth } from "./application/checkServerHealth";
import { makeLogin } from "./application/login";
import { makeRegisterUser } from "./application/registerUser";
import { makeWebCryptoService } from "./infrastructure/crypto/webCryptoService";
import { makeHttpAuthApi } from "./infrastructure/http/authApi";
import { makeHttpHealthGateway } from "./infrastructure/http/healthGateway";
import { makeInMemoryKeyStore } from "./infrastructure/keys/inMemoryKeyStore";
import { App } from "./ui/App";

const cryptoService = makeWebCryptoService();
const authApi = makeHttpAuthApi();
// The real in-memory store (from the unlock/auto-lock task): keys live only
// in its closure and are zeroized on clear() — never localStorage/sessionStorage.
const keyStore = makeInMemoryKeyStore();

const checkServerHealth = makeCheckServerHealth(makeHttpHealthGateway());
const registerUser = makeRegisterUser(cryptoService, authApi);
const login = makeLogin(cryptoService, authApi, keyStore);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App checkServerHealth={checkServerHealth} registerUser={registerUser} login={login} />
  </StrictMode>,
);

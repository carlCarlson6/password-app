// Composition root: the only place adapters and use cases are wired together.

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { makeCheckServerHealth } from "./application/checkServerHealth";
import { makeLogin } from "./application/login";
import type { KeyStore, UnlockedKeys } from "./application/ports";
import { makeRegisterUser } from "./application/registerUser";
import { makeWebCryptoService } from "./infrastructure/crypto/webCryptoService";
import { makeHttpAuthApi } from "./infrastructure/http/authApi";
import { makeHttpHealthGateway } from "./infrastructure/http/healthGateway";
import { App } from "./ui/App";

// Placeholder KeyStore until the real adapter (auto-lock on idle/tab close)
// lands with the unlock-flow task. Keys live only in this closure — never
// localStorage/sessionStorage.
function makePlaceholderKeyStore(): KeyStore {
  let keys: UnlockedKeys | null = null;
  return {
    set(next) {
      keys = next;
    },
    get() {
      return keys;
    },
    clear() {
      keys = null;
    },
  };
}

const cryptoService = makeWebCryptoService();
const authApi = makeHttpAuthApi();
const keyStore = makePlaceholderKeyStore();

const checkServerHealth = makeCheckServerHealth(makeHttpHealthGateway());
const registerUser = makeRegisterUser(cryptoService, authApi);
const login = makeLogin(cryptoService, authApi, keyStore);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App checkServerHealth={checkServerHealth} registerUser={registerUser} login={login} />
  </StrictMode>,
);

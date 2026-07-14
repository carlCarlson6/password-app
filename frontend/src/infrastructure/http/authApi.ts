import type { AuthApi, LoginResponse } from "../../application/ports";
import type { KdfParams } from "../../domain/crypto";

/**
 * Fetch adapter for the AuthApi port — the only place that knows the auth
 * wire paths. Two token rules live here:
 *
 * - The rotating refresh token is an httpOnly cookie: `credentials:
 *   "include"` lets the browser carry it, and this code never reads it.
 * - The short-lived access token stays in this closure — memory only, never
 *   localStorage/sessionStorage. `getAccessToken()` is how future API
 *   clients (vaults, items) will attach it as a bearer header.
 */
export interface HttpAuthApi extends AuthApi {
  getAccessToken(): string | null;
}

export function makeHttpAuthApi(baseUrl = ""): HttpAuthApi {
  let accessToken: string | null = null;

  async function post<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${baseUrl}/api/auth/${path}`, {
      method: "POST",
      credentials: "include",
      headers: body === undefined ? undefined : { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!response.ok) {
      // No detail from the server body: auth errors are deliberately opaque
      // (no user-enumeration), so the status is all there is to say.
      throw new Error(`auth request failed: HTTP ${response.status}`);
    }
    // 201 Created (register) has no body worth parsing.
    return response.status === 201 ? (undefined as T) : ((await response.json()) as T);
  }

  return {
    async prelogin(email) {
      const { kdf } = await post<{ kdf: KdfParams }>("prelogin", { email });
      return kdf;
    },

    async register(request) {
      await post<void>("register", request);
    },

    async login(email, masterPasswordHash) {
      const response = await post<LoginResponse>("login", {
        email,
        masterPasswordHash,
      });
      accessToken = response.accessToken;
      return response;
    },

    async refresh() {
      const { accessToken: fresh } = await post<{ accessToken: string }>("refresh");
      accessToken = fresh;
      return fresh;
    },

    getAccessToken() {
      return accessToken;
    },
  };
}

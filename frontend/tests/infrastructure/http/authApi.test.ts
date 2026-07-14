// Tests for src/infrastructure/http/authApi.ts — the fetch adapter is pinned
// to the auth wire contract (paths, camelCase bodies, cookie credentials).

import { afterEach, describe, expect, it, vi } from "vitest";

import { makeHttpAuthApi } from "../../../src/infrastructure/http/authApi";

interface FakeResponse {
  ok: boolean;
  status: number;
  json: () => Promise<unknown>;
}

function jsonResponse(status: number, body?: unknown): FakeResponse {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => {
      if (body === undefined) throw new Error("no body");
      return body;
    },
  };
}

function stubFetch(response: FakeResponse) {
  const fetchMock = vi.fn(async () => response);
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

const KDF = {
  algorithm: "argon2id",
  memoryKiB: 65536,
  iterations: 3,
  parallelism: 4,
} as const;

describe("makeHttpAuthApi", () => {
  it("prelogin POSTs the email and returns the kdf params", async () => {
    const fetchMock = stubFetch(jsonResponse(200, { kdf: KDF }));

    const kdf = await makeHttpAuthApi().prelogin("user@example.com");

    expect(kdf).toEqual(KDF);
    expect(fetchMock).toHaveBeenCalledWith("/api/auth/prelogin", {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: "user@example.com" }),
    });
  });

  it("register POSTs the full payload and accepts a bodyless 201", async () => {
    const fetchMock = stubFetch(jsonResponse(201));
    const request = {
      email: "user@example.com",
      masterPasswordHash: "aGFzaA==",
      kdf: KDF,
      wrappedUserSymmetricKey: "d3Vzaw==",
      publicKey: "cGs=",
      wrappedPrivateKey: "d3Br",
    };

    await expect(makeHttpAuthApi().register(request)).resolves.toBeUndefined();

    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/auth/register");
    expect(init.credentials).toBe("include");
    expect(JSON.parse(init.body as string)).toEqual(request);
  });

  it("login returns the wrapped keys and keeps the access token in memory", async () => {
    stubFetch(
      jsonResponse(200, {
        accessToken: "jwt-123",
        wrappedUserSymmetricKey: "d3Vzaw==",
        publicKey: "cGs=",
        wrappedPrivateKey: "d3Br",
      }),
    );
    const api = makeHttpAuthApi();
    expect(api.getAccessToken()).toBeNull();

    const response = await api.login("user@example.com", "aGFzaA==");

    expect(response.accessToken).toBe("jwt-123");
    expect(api.getAccessToken()).toBe("jwt-123");
  });

  it("refresh POSTs without a body (the cookie is the credential) and rotates the token", async () => {
    const fetchMock = stubFetch(jsonResponse(200, { accessToken: "jwt-456" }));
    const api = makeHttpAuthApi();

    await expect(api.refresh()).resolves.toBe("jwt-456");

    expect(api.getAccessToken()).toBe("jwt-456");
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/auth/refresh");
    expect(init.credentials).toBe("include");
    expect(init.body).toBeUndefined();
  });

  it("throws on a non-2xx status without leaking detail", async () => {
    stubFetch(jsonResponse(401, { anything: "ignored" }));

    await expect(makeHttpAuthApi().login("user@example.com", "x")).rejects.toThrow(
      "auth request failed: HTTP 401",
    );
  });

  it("prefixes a configured base URL", async () => {
    const fetchMock = stubFetch(jsonResponse(200, { kdf: KDF }));

    await makeHttpAuthApi("https://api.example.com").prelogin("user@example.com");

    const [url] = fetchMock.mock.calls[0] as unknown as [string];
    expect(url).toBe("https://api.example.com/api/auth/prelogin");
  });
});

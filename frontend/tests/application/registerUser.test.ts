// Tests for src/application/registerUser.ts.
//
// Uses the REAL CryptoService (hash-wasm + WebCrypto run under Node) and a
// faked AuthApi port: the point is to prove that everything in the register
// payload can be unwrapped again with keys derived from the password — and
// that nothing secret is in the payload.

import { describe, expect, it } from "vitest";

import type { AuthApi, RegisterRequest } from "../../src/application/ports";
import { makeRegisterUser } from "../../src/application/registerUser";
import {
  decodeBase64,
  decodeBlobBase64,
  encodeBase64,
  type KdfParams,
  type SymmetricKey,
} from "../../src/domain/crypto";
import { makeWebCryptoService } from "../../src/infrastructure/crypto/webCryptoService";

const crypto = makeWebCryptoService();

const EMAIL = "user@example.com";
const PASSWORD = "correct horse battery staple";

/** Cheap Argon2id costs so the suite stays fast (defaults are 64 MiB × 3). */
const TEST_KDF_PARAMS: KdfParams = {
  algorithm: "argon2id",
  memoryKiB: 1024,
  iterations: 2,
  parallelism: 1,
};

const BASE64 = /^[A-Za-z0-9+/]+={0,2}$/;

/** Runs the use case against a capturing fake and returns what was sent. */
async function runRegistration(): Promise<RegisterRequest> {
  let sent: RegisterRequest | null = null;
  const authApi: AuthApi = {
    prelogin: () => Promise.reject(new Error("not used by register")),
    login: () => Promise.reject(new Error("not used by register")),
    refresh: () => Promise.reject(new Error("not used by register")),
    register: async (request) => {
      sent = request;
    },
  };

  await makeRegisterUser(crypto, authApi, TEST_KDF_PARAMS)(EMAIL, PASSWORD);
  expect(sent).not.toBeNull();
  return sent!;
}

describe("registerUser", () => {
  it("sends the wire contract fields, all binary ones as plain base64", async () => {
    const request = await runRegistration();

    expect(request.email).toBe(EMAIL);
    expect(request.kdf).toEqual(TEST_KDF_PARAMS);
    expect(request.masterPasswordHash).toMatch(BASE64);
    expect(request.wrappedUserSymmetricKey).toMatch(BASE64);
    expect(request.publicKey).toMatch(BASE64);
    expect(request.wrappedPrivateKey).toMatch(BASE64);
  }, 30000);

  it("sends the derived login credential — never the password or master key", async () => {
    const request = await runRegistration();

    const masterKey = await crypto.deriveMasterKey(PASSWORD, EMAIL, TEST_KDF_PARAMS);
    const expectedHash = await crypto.deriveMasterPasswordHash(masterKey, PASSWORD);

    expect(request.masterPasswordHash).toBe(encodeBase64(expectedHash));
    expect(request.masterPasswordHash).not.toBe(encodeBase64(masterKey));
    expect(JSON.stringify(request)).not.toContain(PASSWORD);
  }, 30000);

  it("wraps keys that round-trip: stretched key → USK → private key → RSA", async () => {
    const request = await runRegistration();

    // Re-derive the wrapping key exactly as a fresh login would.
    const masterKey = await crypto.deriveMasterKey(PASSWORD, EMAIL, TEST_KDF_PARAMS);
    const stretched = await crypto.stretchMasterKey(masterKey);

    // Unwrap the User Symmetric Key with the stretched master key.
    const usk = (await crypto.decrypt(
      stretched,
      decodeBlobBase64(request.wrappedUserSymmetricKey),
    )) as SymmetricKey;
    expect(usk).toHaveLength(32);

    // Unwrap the private key with the USK, then prove the keypair matches:
    // what the public key encrypts, the unwrapped private key decrypts.
    const privateKeyPkcs8 = await crypto.decrypt(
      usk,
      decodeBlobBase64(request.wrappedPrivateKey),
    );
    const secret = crypto.generateSymmetricKey();
    const rsaWrapped = await crypto.rsaEncrypt(decodeBase64(request.publicKey), secret);
    const rsaUnwrapped = await crypto.rsaDecrypt(privateKeyPkcs8, rsaWrapped);
    expect(rsaUnwrapped).toEqual(secret);
  }, 30000);

  it("rejects (and does not swallow) an API failure", async () => {
    const authApi: AuthApi = {
      prelogin: () => Promise.reject(new Error("not used")),
      login: () => Promise.reject(new Error("not used")),
      refresh: () => Promise.reject(new Error("not used")),
      register: () => Promise.reject(new Error("email already taken")),
    };

    await expect(
      makeRegisterUser(crypto, authApi, TEST_KDF_PARAMS)(EMAIL, PASSWORD),
    ).rejects.toThrow("email already taken");
  }, 30000);
});

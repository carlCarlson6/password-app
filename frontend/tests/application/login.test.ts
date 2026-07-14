// Tests for src/application/login.ts.
//
// A fake "server" is prepared with the REAL CryptoService, exactly as
// registration would have stored it (wrapped USK, wrapped private key,
// login-hash). Login must unwrap everything locally into the KeyStore.

import { beforeAll, describe, expect, it } from "vitest";

import type {
  AuthApi,
  KeyStore,
  LoginResponse,
  UnlockedKeys,
} from "../../src/application/ports";
import { makeLogin } from "../../src/application/login";
import {
  encodeBase64,
  encodeBlobBase64,
  type KdfParams,
  type RsaKeyPair,
  type SymmetricKey,
} from "../../src/domain/crypto";
import { makeWebCryptoService } from "../../src/infrastructure/crypto/webCryptoService";

const crypto = makeWebCryptoService();

const EMAIL = "user@example.com";
const PASSWORD = "correct horse battery staple";

const TEST_KDF_PARAMS: KdfParams = {
  algorithm: "argon2id",
  memoryKiB: 1024,
  iterations: 2,
  parallelism: 1,
};

/** Trivial in-memory stub of the KeyStore port (real adapter is a separate task). */
function makeStubKeyStore(): KeyStore {
  let keys: UnlockedKeys | null = null;
  return {
    set: (next) => {
      keys = next;
    },
    get: () => keys,
    clear: () => {
      keys = null;
    },
  };
}

// What the server stored at signup — computed once, with the real crypto.
let userSymmetricKey: SymmetricKey;
let keyPair: RsaKeyPair;
let expectedHash: string;
let loginResponse: LoginResponse;

beforeAll(async () => {
  const masterKey = await crypto.deriveMasterKey(PASSWORD, EMAIL, TEST_KDF_PARAMS);
  const stretched = await crypto.stretchMasterKey(masterKey);
  expectedHash = encodeBase64(
    await crypto.deriveMasterPasswordHash(masterKey, PASSWORD),
  );

  userSymmetricKey = crypto.generateSymmetricKey();
  keyPair = await crypto.generateRsaKeyPair();

  loginResponse = {
    accessToken: "test-access-token",
    wrappedUserSymmetricKey: encodeBlobBase64(
      await crypto.encrypt(stretched, userSymmetricKey),
    ),
    publicKey: encodeBase64(keyPair.publicKeySpki),
    wrappedPrivateKey: encodeBlobBase64(
      await crypto.encrypt(userSymmetricKey, keyPair.privateKeyPkcs8),
    ),
  };
}, 60000);

/** Fake server: prelogin serves the KDF params, login checks the credential. */
function makeFakeAuthApi(sent: { credential?: string } = {}): AuthApi {
  return {
    prelogin: async (email) => {
      expect(email).toBe(EMAIL);
      return TEST_KDF_PARAMS;
    },
    login: async (_email, masterPasswordHash) => {
      sent.credential = masterPasswordHash;
      if (masterPasswordHash !== expectedHash) {
        throw new Error("invalid credentials");
      }
      return loginResponse;
    },
    register: () => Promise.reject(new Error("not used by login")),
    refresh: () => Promise.reject(new Error("not used by login")),
  };
}

describe("login", () => {
  it("unwraps the USK and private key into the key store", async () => {
    const keyStore = makeStubKeyStore();

    await makeLogin(crypto, makeFakeAuthApi(), keyStore)(EMAIL, PASSWORD);

    const unlocked = keyStore.get();
    expect(unlocked).not.toBeNull();
    expect(unlocked!.userSymmetricKey).toEqual(userSymmetricKey);
    expect(unlocked!.privateKeyPkcs8).toEqual(keyPair.privateKeyPkcs8);
    expect(unlocked!.publicKeySpki).toEqual(keyPair.publicKeySpki);
  });

  it("sends the derived credential from prelogin params — never the password", async () => {
    const sent: { credential?: string } = {};

    await makeLogin(crypto, makeFakeAuthApi(sent), makeStubKeyStore())(
      EMAIL,
      PASSWORD,
    );

    expect(sent.credential).toBe(expectedHash);
    expect(sent.credential).not.toContain(PASSWORD);
  });

  it("rejects a wrong password and leaves the key store locked", async () => {
    const keyStore = makeStubKeyStore();

    await expect(
      makeLogin(crypto, makeFakeAuthApi(), keyStore)(EMAIL, "wrong password"),
    ).rejects.toThrow("invalid credentials");
    expect(keyStore.get()).toBeNull();
  });

  it("rejects tampered wrapped keys instead of storing junk", async () => {
    // Same credential, but the server returns someone else's wrapped USK:
    // the AES-GCM tag check must make the unwrap throw.
    const tamperedResponse: LoginResponse = {
      ...loginResponse,
      wrappedUserSymmetricKey: encodeBlobBase64(
        await crypto.encrypt(crypto.generateSymmetricKey(), userSymmetricKey),
      ),
    };
    const authApi: AuthApi = {
      ...makeFakeAuthApi(),
      login: async () => tamperedResponse,
    };
    const keyStore = makeStubKeyStore();

    await expect(
      makeLogin(crypto, authApi, keyStore)(EMAIL, PASSWORD),
    ).rejects.toThrow();
    expect(keyStore.get()).toBeNull();
  });
});

// Tests for src/application/unlockVault.ts — full round-trip against the
// real CryptoService (WebCrypto + Argon2id WASM), no crypto stubs.

import { beforeAll, beforeEach, describe, expect, it } from "vitest";

import { makeUnlockVault, type UnlockContext } from "../../src/application/unlockVault";
import type { KeyStore } from "../../src/application/ports/keyStore";
import type { KdfParams, RsaKeyPair, SymmetricKey } from "../../src/domain/crypto";
import { makeWebCryptoService } from "../../src/infrastructure/crypto/webCryptoService";
import { makeInMemoryKeyStore } from "../../src/infrastructure/keys/inMemoryKeyStore";

const cryptoService = makeWebCryptoService();

const PASSWORD = "correct horse battery staple";
const EMAIL = "user@example.com";

/** Reduced Argon2id cost so the suite stays fast (same as the crypto tests). */
const TEST_KDF_PARAMS: KdfParams = {
  algorithm: "argon2id",
  memoryKiB: 1024,
  iterations: 2,
  parallelism: 1,
};

// "Signup" once: generate the real key hierarchy and wrap it exactly the way
// the register flow will, so unlock exercises the true contract.
let userSymmetricKey: SymmetricKey;
let keyPair: RsaKeyPair;
let context: UnlockContext;

beforeAll(async () => {
  const masterKey = await cryptoService.deriveMasterKey(PASSWORD, EMAIL, TEST_KDF_PARAMS);
  const stretchedKey = await cryptoService.stretchMasterKey(masterKey);
  userSymmetricKey = cryptoService.generateSymmetricKey();
  keyPair = await cryptoService.generateRsaKeyPair();

  context = {
    email: EMAIL,
    kdfParams: TEST_KDF_PARAMS,
    wrappedUserSymmetricKey: await cryptoService.encrypt(stretchedKey, userSymmetricKey),
    wrappedPrivateKey: await cryptoService.encrypt(
      userSymmetricKey,
      keyPair.privateKeyPkcs8,
    ),
  };
}, 30000);

describe("unlockVault", () => {
  let keyStore: KeyStore;
  let unlockVault: ReturnType<typeof makeUnlockVault>;

  beforeEach(() => {
    keyStore = makeInMemoryKeyStore();
    unlockVault = makeUnlockVault(cryptoService, keyStore);
  });

  it("repopulates the key store with the correct password", async () => {
    const unlocked = await unlockVault(context, PASSWORD);

    expect(unlocked).toBe(true);
    const keys = keyStore.get();
    expect(keys).not.toBeNull();
    expect(keys!.userSymmetricKey).toEqual(userSymmetricKey);
    expect(keys!.privateKeyPkcs8).toEqual(keyPair.privateKeyPkcs8);
  });

  it("fails cleanly on a wrong password, leaving the store locked", async () => {
    const unlocked = await unlockVault(context, "not the master password");

    expect(unlocked).toBe(false);
    expect(keyStore.get()).toBeNull();
  });

  it("re-unlocks after a lock using only the retained wrapped keys", async () => {
    await unlockVault(context, PASSWORD);
    keyStore.clear(); // lock (zeroizes the unwrapped copies)

    const unlocked = await unlockVault(context, PASSWORD);

    expect(unlocked).toBe(true);
    expect(keyStore.get()!.userSymmetricKey).toEqual(userSymmetricKey);
    expect(keyStore.get()!.privateKeyPkcs8).toEqual(keyPair.privateKeyPkcs8);
  });

  it("a wrong attempt does not clobber an already-unlocked store", async () => {
    await unlockVault(context, PASSWORD);

    const retry = await unlockVault(context, "wrong");

    expect(retry).toBe(false);
    expect(keyStore.get()).not.toBeNull();
    expect(keyStore.get()!.userSymmetricKey).toEqual(userSymmetricKey);
  });

  it("unwrapped keys still decrypt real data (end-to-end sanity)", async () => {
    await unlockVault(context, PASSWORD);
    const keys = keyStore.get()!;

    const secret = new TextEncoder().encode("hunter2");
    const blob = await cryptoService.encrypt(userSymmetricKey, secret);
    const decrypted = await cryptoService.decrypt(keys.userSymmetricKey, blob);

    expect(decrypted).toEqual(secret);
  });
});

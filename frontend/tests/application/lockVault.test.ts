// Tests for src/application/lockVault.ts.

import { describe, expect, it } from "vitest";

import { makeLockVault } from "../../src/application/lockVault";
import type { SymmetricKey } from "../../src/domain/crypto";
import { makeInMemoryKeyStore } from "../../src/infrastructure/keys/inMemoryKeyStore";

describe("lockVault", () => {
  it("clears the key store", () => {
    const keyStore = makeInMemoryKeyStore();
    keyStore.set({
      userSymmetricKey: new Uint8Array(32).fill(1) as SymmetricKey,
      publicKeySpki: new Uint8Array(16).fill(3),
      privateKeyPkcs8: new Uint8Array(64).fill(2),
    });

    makeLockVault(keyStore)();

    expect(keyStore.get()).toBeNull();
  });

  it("is idempotent on an already-locked store", () => {
    const keyStore = makeInMemoryKeyStore();
    const lockVault = makeLockVault(keyStore);

    lockVault();
    lockVault();

    expect(keyStore.get()).toBeNull();
  });
});

// Tests for src/infrastructure/keys/inMemoryKeyStore.ts.

import { describe, expect, it } from "vitest";

import type { UnlockedKeys } from "../../../src/application/ports/keyStore";
import type { SymmetricKey } from "../../../src/domain/crypto";
import { makeInMemoryKeyStore } from "../../../src/infrastructure/keys/inMemoryKeyStore";

function someKeys(): UnlockedKeys {
  return {
    userSymmetricKey: new Uint8Array(32).fill(7) as SymmetricKey,
    publicKeySpki: new Uint8Array(16).fill(4),
    privateKeyPkcs8: new Uint8Array(64).fill(9),
  };
}

describe("makeInMemoryKeyStore", () => {
  it("starts locked", () => {
    expect(makeInMemoryKeyStore().get()).toBeNull();
  });

  it("returns what was set", () => {
    const store = makeInMemoryKeyStore();
    const keys = someKeys();

    store.set(keys);

    expect(store.get()).toBe(keys);
  });

  it("clear() locks the store", () => {
    const store = makeInMemoryKeyStore();
    store.set(someKeys());

    store.clear();

    expect(store.get()).toBeNull();
  });

  it("clear() zeroizes the held key bytes", () => {
    const store = makeInMemoryKeyStore();
    const keys = someKeys();
    store.set(keys);

    store.clear();

    expect(keys.userSymmetricKey.every((byte) => byte === 0)).toBe(true);
    expect(keys.privateKeyPkcs8.every((byte) => byte === 0)).toBe(true);
  });

  it("clear() on a locked store is a no-op", () => {
    const store = makeInMemoryKeyStore();

    expect(() => store.clear()).not.toThrow();
    expect(store.get()).toBeNull();
  });

  it("can be set again after clearing (re-unlock)", () => {
    const store = makeInMemoryKeyStore();
    store.set(someKeys());
    store.clear();

    const fresh = someKeys();
    store.set(fresh);

    expect(store.get()).toBe(fresh);
  });
});

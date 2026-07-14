import type { KeyStore, UnlockedKeys } from "../../application/ports/keyStore";

/**
 * The only place unwrapped keys are allowed to rest: a closure variable.
 * Nothing here (or anywhere else) may write key material to localStorage,
 * sessionStorage, IndexedDB, cookies, or any serialized form — closing the
 * tab must be equivalent to locking.
 */
export function makeInMemoryKeyStore(): KeyStore {
  let keys: UnlockedKeys | null = null;

  return {
    set(next) {
      keys = next;
    },

    get() {
      return keys;
    },

    clear() {
      if (keys !== null) {
        // Best-effort zeroization: JS can't guarantee the GC never copied the
        // bytes, but overwriting the live buffers shrinks the window in which
        // a heap dump reveals key material.
        keys.userSymmetricKey.fill(0);
        keys.privateKeyPkcs8.fill(0);
        keys = null;
      }
    },
  };
}

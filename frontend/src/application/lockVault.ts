import type { KeyStore } from "./ports/keyStore";

/**
 * Locking is simply forgetting: clear the in-memory key store (which
 * best-effort zeroizes the key bytes). The wrapped keys retained for
 * re-unlock are ciphertext and stay untouched.
 */
export function makeLockVault(keyStore: KeyStore) {
  return function lockVault(): void {
    keyStore.clear();
  };
}

export type LockVault = ReturnType<typeof makeLockVault>;

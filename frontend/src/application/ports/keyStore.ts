import type { SymmetricKey } from "../../domain/crypto";

/**
 * The unwrapped key material held while the vault is unlocked — exactly what
 * unwrapping the server's wrapped keys produces (see README "Key hierarchy"):
 * the User Symmetric Key and the RSA private key it wraps.
 */
export interface UnlockedKeys {
  readonly userSymmetricKey: SymmetricKey;
  readonly privateKeyPkcs8: Uint8Array;
}

/**
 * Driven port: where unwrapped keys live while the vault is unlocked.
 * Implemented in `infrastructure/` strictly in memory — keys must NEVER touch
 * localStorage/sessionStorage/IndexedDB or be serialized in any form.
 */
export interface KeyStore {
  set(keys: UnlockedKeys): void;
  /** `null` while locked. */
  get(): UnlockedKeys | null;
  /** Locks: drops (and best-effort zeroizes) the held keys. */
  clear(): void;
}

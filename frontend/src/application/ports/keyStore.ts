import type { SymmetricKey } from "../../domain/crypto";

/**
 * The unwrapped key material held while the vault is unlocked — exactly what
 * unwrapping the server's wrapped keys produces (see README "Key hierarchy").
 * Everything here must live in memory only — never localStorage/sessionStorage.
 */
export interface UnlockedKeys {
  /** The User Symmetric Key, unwrapped with the stretched master key. */
  readonly userSymmetricKey: SymmetricKey;
  /** RSA-OAEP-2048 public key (SPKI bytes) — not secret; along for convenience. */
  readonly publicKeySpki: Uint8Array;
  /** RSA-OAEP-2048 private key (PKCS#8 bytes), unwrapped with the USK. */
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

import type { SymmetricKey } from "../../domain/crypto";

/**
 * The unlocked key material a successful login produces. Everything here is
 * secret (the public key travels along for convenience) and must live in
 * memory only — never localStorage/sessionStorage.
 */
export interface UnlockedKeys {
  /** The User Symmetric Key, unwrapped with the stretched master key. */
  readonly userSymmetricKey: SymmetricKey;
  /** RSA-OAEP-2048 public key (SPKI bytes). */
  readonly publicKeySpki: Uint8Array;
  /** RSA-OAEP-2048 private key (PKCS#8 bytes), unwrapped with the USK. */
  readonly privateKeyPkcs8: Uint8Array;
}

/**
 * Driven port: where the unlocked keys live while the vault is open.
 * The real adapter (in-memory holder + auto-lock on idle/tab close) is a
 * separate deliverable; login only depends on this seam.
 */
export interface KeyStore {
  set(keys: UnlockedKeys): void;
  /** `null` while locked. */
  get(): UnlockedKeys | null;
  clear(): void;
}

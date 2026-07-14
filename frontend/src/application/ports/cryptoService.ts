import type {
  AesKey,
  EncryptedBlob,
  KdfParams,
  MasterKey,
  MasterPasswordHash,
  RsaKeyPair,
  StretchedMasterKey,
  SymmetricKey,
} from "../../domain/crypto";

/**
 * Driven port: every cryptographic primitive the client needs, in one place.
 * Implemented in `infrastructure/crypto/` (WebCrypto + Argon2id WASM); the UI
 * and use cases never touch WebCrypto directly, so the implementation stays
 * swappable (e.g., OPAQUE later) and testable.
 *
 * Key hierarchy (README "Security model"): master password + email
 * → Master Key → (HKDF) Stretched Master Key → wraps the User Symmetric Key,
 * which wraps the RSA private key and the per-vault Vault Keys.
 */
export interface CryptoService {
  /** Argon2id(master password, salt derived from normalized email). */
  deriveMasterKey(
    masterPassword: string,
    email: string,
    params: KdfParams,
  ): Promise<MasterKey>;

  /**
   * The login credential: a cheap one-way Argon2id of the Master Key keyed by
   * the password. Safe to send — the server re-hashes it before storing.
   */
  deriveMasterPasswordHash(
    masterKey: MasterKey,
    masterPassword: string,
  ): Promise<MasterPasswordHash>;

  /** HKDF-SHA256 expansion of the Master Key into the key-wrapping key. */
  stretchMasterKey(masterKey: MasterKey): Promise<StretchedMasterKey>;

  /** Fresh random 256-bit key (User Symmetric Key at signup, Vault Keys later). */
  generateSymmetricKey(): SymmetricKey;

  /** RSA-OAEP-2048 keypair, generated at signup so sharing (v2) needs no migration. */
  generateRsaKeyPair(): Promise<RsaKeyPair>;

  /** AES-256-GCM with a fresh random IV per call. */
  encrypt(key: AesKey, plaintext: Uint8Array): Promise<EncryptedBlob>;

  /** Rejects (throws) if the key is wrong or the blob was tampered with. */
  decrypt(key: AesKey, blob: EncryptedBlob): Promise<Uint8Array>;

  /** RSA-OAEP-SHA256 — wraps small payloads (key material) to a public key. */
  rsaEncrypt(publicKeySpki: Uint8Array, data: Uint8Array): Promise<Uint8Array>;

  rsaDecrypt(privateKeyPkcs8: Uint8Array, data: Uint8Array): Promise<Uint8Array>;
}

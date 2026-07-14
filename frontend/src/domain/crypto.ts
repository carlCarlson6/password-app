/**
 * Pure crypto value types for the client "Credentials" domain.
 *
 * No WebCrypto, no I/O — just the vocabulary of the key hierarchy
 * (see README "Security model") plus the wire format for ciphertext.
 * The branded types make it a compile error to, say, encrypt with a
 * raw Master Key where the Stretched Master Key is required.
 */

/** Argon2id parameters. Served by `/api/auth/prelogin`; pinned here for signup. */
export interface KdfParams {
  readonly algorithm: "argon2id";
  /** Memory cost in KiB. */
  readonly memoryKiB: number;
  /** Time cost (passes over memory). */
  readonly iterations: number;
  /** Lanes/threads. */
  readonly parallelism: number;
}

/** Signup-time defaults (README: m=64MiB, t=3, p=4). */
export const DEFAULT_KDF_PARAMS: KdfParams = {
  algorithm: "argon2id",
  memoryKiB: 64 * 1024,
  iterations: 3,
  parallelism: 4,
};

// Brands are phantom types: they exist only for the compiler, the runtime
// value is a plain Uint8Array. Only the crypto adapter should cast into them.
declare const brand: unique symbol;
type Branded<Name extends string> = Uint8Array & { readonly [brand]: Name };

/** Argon2id(master password, salt=email). Never leaves the client. */
export type MasterKey = Branded<"MasterKey">;

/** HKDF-SHA256 expansion of the Master Key; wraps the User Symmetric Key. */
export type StretchedMasterKey = Branded<"StretchedMasterKey">;

/** Random 256-bit AES key (User Symmetric Key, Vault Keys). */
export type SymmetricKey = Branded<"SymmetricKey">;

/** Login credential sent to the server (which re-hashes it at rest). */
export type MasterPasswordHash = Branded<"MasterPasswordHash">;

/** Keys the AES-256-GCM operations accept. */
export type AesKey = StretchedMasterKey | SymmetricKey;

/** RSA-OAEP-2048 keypair in standard export formats (raw bytes, unencrypted). */
export interface RsaKeyPair {
  readonly publicKeySpki: Uint8Array;
  readonly privateKeyPkcs8: Uint8Array;
}

/**
 * One AES-256-GCM encryption: fresh random IV + ciphertext (GCM tag appended).
 * Serialized this is the opaque `CipherBlob` the server stores.
 */
export interface EncryptedBlob {
  readonly iv: Uint8Array;
  readonly ciphertext: Uint8Array;
}

/** Wire format: `v1.<base64 iv>.<base64 ciphertext>`. */
const BLOB_VERSION = "v1";

export function serializeBlob(blob: EncryptedBlob): string {
  return `${BLOB_VERSION}.${encodeBase64(blob.iv)}.${encodeBase64(blob.ciphertext)}`;
}

export function parseBlob(serialized: string): EncryptedBlob {
  const parts = serialized.split(".");
  if (parts.length !== 3 || parts[0] !== BLOB_VERSION) {
    throw new Error(`malformed cipher blob (expected "${BLOB_VERSION}.<iv>.<ct>")`);
  }
  return { iv: decodeBase64(parts[1]), ciphertext: decodeBase64(parts[2]) };
}

/**
 * API transport for wrapped keys: the auth wire contract wants plain base64,
 * so the canonical versioned blob string (`v1.<iv>.<ct>`) is base64-wrapped
 * whole. The blob stays versioned and stays opaque to the server.
 */
export function encodeBlobBase64(blob: EncryptedBlob): string {
  // serializeBlob output is pure ASCII, so btoa is safe here.
  return btoa(serializeBlob(blob));
}

export function decodeBlobBase64(text: string): EncryptedBlob {
  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(text)) {
    throw new Error("malformed base64");
  }
  return parseBlob(atob(text));
}

// btoa/atob work on "binary strings"; these helpers hide that quirk.
// (Available in browsers and Node ≥ 16 — no Buffer, so the domain stays pure.)
export function encodeBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export function decodeBase64(text: string): Uint8Array {
  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(text)) {
    throw new Error("malformed base64");
  }
  const binary = atob(text);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

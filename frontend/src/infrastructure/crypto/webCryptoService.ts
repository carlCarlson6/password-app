import { argon2id } from "hash-wasm";

import type { CryptoService } from "../../application/ports";
import type {
  AesKey,
  EncryptedBlob,
  MasterKey,
  MasterPasswordHash,
  RsaKeyPair,
  StretchedMasterKey,
  SymmetricKey,
} from "../../domain/crypto";

/**
 * The one adapter allowed to touch WebCrypto and the Argon2id WASM
 * (hash-wasm). Everything here runs in the browser — no key material ever
 * leaves this process, and nothing is persisted.
 *
 * ⚠️ Every constant below (salts, info strings, parameters) is part of the
 * key derivation contract: changing any of them locks every existing user
 * out of their vault. The pinned test vectors in
 * `tests/infrastructure/crypto/` exist to make such a change impossible to
 * miss.
 */

const textEncoder = new TextEncoder();

/** Domain-separation label for the Master Key → Stretched Master Key HKDF. */
const STRETCH_INFO = "password-app/stretched-master-key/v1";

/**
 * Login-hash Argon2id cost. Deliberately cheap: the input is already a
 * 256-bit key, so this pass only needs to be one-way (Bitwarden uses a
 * single PBKDF2 iteration here); the server re-hashes with full cost at rest.
 */
const LOGIN_HASH_COST = { iterations: 1, memoryKiB: 1024, parallelism: 1 };

const AES_GCM_IV_BYTES = 12;

const RSA_OAEP = { name: "RSA-OAEP", hash: "SHA-256" } as const;

/** Case/whitespace in the email must never change the derived key. */
function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

async function sha256(data: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", data));
}

async function runArgon2id(
  password: string | Uint8Array,
  salt: Uint8Array,
  cost: { iterations: number; memoryKiB: number; parallelism: number },
): Promise<Uint8Array> {
  return argon2id({
    password,
    salt,
    iterations: cost.iterations,
    memorySize: cost.memoryKiB,
    parallelism: cost.parallelism,
    hashLength: 32,
    outputType: "binary",
  });
}

async function importAesKey(key: AesKey, usage: KeyUsage): Promise<CryptoKey> {
  return crypto.subtle.importKey("raw", key, "AES-GCM", false, [usage]);
}

export function makeWebCryptoService(): CryptoService {
  return {
    async deriveMasterKey(masterPassword, email, params) {
      // Argon2 requires a salt of ≥ 8 bytes and short emails ("a@b.c") exist,
      // so the salt is SHA-256(normalized email) — still deterministic per email.
      const salt = await sha256(textEncoder.encode(normalizeEmail(email)));
      const key = await runArgon2id(masterPassword, salt, params);
      return key as MasterKey;
    },

    async deriveMasterPasswordHash(masterKey, masterPassword) {
      // README: MPH = Argon2id(MK, password). The password enters as the salt
      // (hashed to satisfy Argon2's minimum salt length).
      const salt = await sha256(textEncoder.encode(masterPassword));
      const hash = await runArgon2id(masterKey, salt, LOGIN_HASH_COST);
      return hash as MasterPasswordHash;
    },

    async stretchMasterKey(masterKey) {
      const ikm = await crypto.subtle.importKey("raw", masterKey, "HKDF", false, [
        "deriveBits",
      ]);
      const bits = await crypto.subtle.deriveBits(
        {
          name: "HKDF",
          hash: "SHA-256",
          salt: new Uint8Array(0),
          info: textEncoder.encode(STRETCH_INFO),
        },
        ikm,
        256,
      );
      return new Uint8Array(bits) as StretchedMasterKey;
    },

    generateSymmetricKey() {
      return crypto.getRandomValues(new Uint8Array(32)) as SymmetricKey;
    },

    async generateRsaKeyPair() {
      const pair = await crypto.subtle.generateKey(
        { ...RSA_OAEP, modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]) },
        true, // extractable: the private key is exported, wrapped under the USK, stored
        ["encrypt", "decrypt"],
      );
      const [publicKeySpki, privateKeyPkcs8] = await Promise.all([
        crypto.subtle.exportKey("spki", pair.publicKey),
        crypto.subtle.exportKey("pkcs8", pair.privateKey),
      ]);
      return {
        publicKeySpki: new Uint8Array(publicKeySpki),
        privateKeyPkcs8: new Uint8Array(privateKeyPkcs8),
      } satisfies RsaKeyPair;
    },

    async encrypt(key, plaintext) {
      // GCM's hard rule: never reuse an IV under the same key.
      const iv = crypto.getRandomValues(new Uint8Array(AES_GCM_IV_BYTES));
      const cryptoKey = await importAesKey(key, "encrypt");
      const ciphertext = await crypto.subtle.encrypt(
        { name: "AES-GCM", iv },
        cryptoKey,
        plaintext,
      );
      return { iv, ciphertext: new Uint8Array(ciphertext) } satisfies EncryptedBlob;
    },

    async decrypt(key, blob) {
      const cryptoKey = await importAesKey(key, "decrypt");
      // WebCrypto verifies the GCM tag and rejects on any mismatch.
      const plaintext = await crypto.subtle.decrypt(
        { name: "AES-GCM", iv: blob.iv },
        cryptoKey,
        blob.ciphertext,
      );
      return new Uint8Array(plaintext);
    },

    async rsaEncrypt(publicKeySpki, data) {
      const key = await crypto.subtle.importKey("spki", publicKeySpki, RSA_OAEP, false, [
        "encrypt",
      ]);
      return new Uint8Array(await crypto.subtle.encrypt(RSA_OAEP, key, data));
    },

    async rsaDecrypt(privateKeyPkcs8, data) {
      const key = await crypto.subtle.importKey(
        "pkcs8",
        privateKeyPkcs8,
        RSA_OAEP,
        false,
        ["decrypt"],
      );
      return new Uint8Array(await crypto.subtle.decrypt(RSA_OAEP, key, data));
    },
  };
}

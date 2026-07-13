// Tests for src/infrastructure/crypto/webCryptoService.ts.
//
// The hex values below are PINNED TEST VECTORS, generated once from this
// implementation and frozen. If one of these tests starts failing, the key
// derivation contract changed — that locks real users out of their vaults.
// Never "fix" a vector to make the suite green; fix the regression instead.

import { describe, expect, it } from "vitest";

import type { CryptoService } from "../../../src/application/ports";
import {
  DEFAULT_KDF_PARAMS,
  parseBlob,
  serializeBlob,
  type KdfParams,
  type MasterKey,
  type StretchedMasterKey,
  type SymmetricKey,
} from "../../../src/domain/crypto";
import { makeWebCryptoService } from "../../../src/infrastructure/crypto/webCryptoService";

const service: CryptoService = makeWebCryptoService();

const PASSWORD = "correct horse battery staple";
// Deliberately messy: normalization (trim + lowercase) is part of the contract.
const EMAIL = "  User@Example.COM ";

/** Reduced cost so most vector tests run in milliseconds. */
const TEST_KDF_PARAMS: KdfParams = {
  algorithm: "argon2id",
  memoryKiB: 1024,
  iterations: 2,
  parallelism: 1,
};

const PINNED = {
  masterKey: "35dccab354151042368f592f3df2242d67bd2090adb747c67f901d649193855b",
  masterKeyDefaultParams:
    "26034d0eaf9c2631caec3e6cc7f47603bee30bf3d86f8d79968d05fa3245f702",
  masterPasswordHash:
    "d775cf27e7e514575a3a4c2aaf4d06f7b26fdd63a9b8275c8effd07676f2636f",
  stretchedMasterKey:
    "07f0de0d124191254e1d113262ca152f1c37ee8f80f265edd166643bdf609407",
  // AES-256-GCM of "attack at dawn" under the stretched master key above.
  blob: "v1.lmHVgKGdJOfw4SnS.QJA1URt0M0bMdzt7dmjluRNpsmJehWNUN6YULx87",
};

const hex = (bytes: Uint8Array) =>
  [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");

const fromHex = (text: string) =>
  new Uint8Array(text.match(/.{2}/g)!.map((b) => parseInt(b, 16)));

const pinnedMasterKey = () => fromHex(PINNED.masterKey) as MasterKey;
const pinnedStretchedKey = () => fromHex(PINNED.stretchedMasterKey) as StretchedMasterKey;

describe("deriveMasterKey (Argon2id)", () => {
  it("matches the pinned vector", async () => {
    const key = await service.deriveMasterKey(PASSWORD, EMAIL, TEST_KDF_PARAMS);
    expect(hex(key)).toBe(PINNED.masterKey);
  });

  it("matches the pinned vector at the real signup cost", async () => {
    const key = await service.deriveMasterKey(PASSWORD, EMAIL, DEFAULT_KDF_PARAMS);
    expect(hex(key)).toBe(PINNED.masterKeyDefaultParams);
  }, 30000);

  it("normalizes the email, so case and whitespace do not change the key", async () => {
    const key = await service.deriveMasterKey(
      PASSWORD,
      "user@example.com",
      TEST_KDF_PARAMS,
    );
    expect(hex(key)).toBe(PINNED.masterKey);
  });

  it("derives a different key for a different email", async () => {
    const key = await service.deriveMasterKey(
      PASSWORD,
      "other@example.com",
      TEST_KDF_PARAMS,
    );
    expect(hex(key)).not.toBe(PINNED.masterKey);
  });
});

describe("deriveMasterPasswordHash", () => {
  it("matches the pinned vector", async () => {
    const hash = await service.deriveMasterPasswordHash(pinnedMasterKey(), PASSWORD);
    expect(hex(hash)).toBe(PINNED.masterPasswordHash);
  });

  it("never equals the master key itself", async () => {
    // The hash goes to the server; the master key must not be recoverable from it.
    const hash = await service.deriveMasterPasswordHash(pinnedMasterKey(), PASSWORD);
    expect(hex(hash)).not.toBe(PINNED.masterKey);
  });
});

describe("stretchMasterKey (HKDF-SHA256)", () => {
  it("matches the pinned vector", async () => {
    const stretched = await service.stretchMasterKey(pinnedMasterKey());
    expect(hex(stretched)).toBe(PINNED.stretchedMasterKey);
  });
});

describe("AES-256-GCM encrypt/decrypt", () => {
  it("decrypts the pinned blob", async () => {
    const plaintext = await service.decrypt(
      pinnedStretchedKey(),
      parseBlob(PINNED.blob),
    );
    expect(new TextDecoder().decode(plaintext)).toBe("attack at dawn");
  });

  it("round-trips through the serialized wire format", async () => {
    const key = service.generateSymmetricKey();
    const message = new TextEncoder().encode("s3cr3t note");

    const blob = await service.encrypt(key, message);
    const decrypted = await service.decrypt(key, parseBlob(serializeBlob(blob)));

    expect(decrypted).toEqual(message);
  });

  it("uses a fresh IV per call, so equal plaintexts encrypt differently", async () => {
    const key = service.generateSymmetricKey();
    const message = new TextEncoder().encode("same input");

    const first = await service.encrypt(key, message);
    const second = await service.encrypt(key, message);

    expect(hex(first.iv)).not.toBe(hex(second.iv));
    expect(hex(first.ciphertext)).not.toBe(hex(second.ciphertext));
  });

  it("rejects decryption with the wrong key", async () => {
    const blob = await service.encrypt(
      service.generateSymmetricKey(),
      new TextEncoder().encode("data"),
    );
    await expect(service.decrypt(service.generateSymmetricKey(), blob)).rejects.toThrow();
  });

  it("rejects a tampered ciphertext (GCM tag check)", async () => {
    const key = service.generateSymmetricKey();
    const blob = await service.encrypt(key, new TextEncoder().encode("data"));

    const tampered = new Uint8Array(blob.ciphertext);
    tampered[0] ^= 0xff;

    await expect(
      service.decrypt(key, { iv: blob.iv, ciphertext: tampered }),
    ).rejects.toThrow();
  });
});

describe("generateSymmetricKey", () => {
  it("returns 32 random bytes, different every call", () => {
    const first = service.generateSymmetricKey();
    const second = service.generateSymmetricKey();

    expect(first).toHaveLength(32);
    expect(hex(first)).not.toBe(hex(second));
  });
});

describe("RSA-OAEP-2048 keypair", () => {
  it("generates exportable keys and round-trips a wrapped key", async () => {
    const pair = await service.generateRsaKeyPair();

    // 294 bytes is the fixed SPKI size of an RSA-2048 key with e=65537 —
    // pins the modulus length without parsing ASN.1.
    expect(pair.publicKeySpki).toHaveLength(294);
    expect(pair.privateKeyPkcs8.length).toBeGreaterThan(1000);

    const vaultKey = service.generateSymmetricKey();
    const wrapped = await service.rsaEncrypt(pair.publicKeySpki, vaultKey);
    const unwrapped = await service.rsaDecrypt(pair.privateKeyPkcs8, wrapped);

    expect(unwrapped).toEqual(vaultKey as SymmetricKey);
  }, 30000);

  it("rejects unwrapping with a different private key", async () => {
    const [alice, mallory] = await Promise.all([
      service.generateRsaKeyPair(),
      service.generateRsaKeyPair(),
    ]);
    const wrapped = await service.rsaEncrypt(
      alice.publicKeySpki,
      service.generateSymmetricKey(),
    );
    await expect(service.rsaDecrypt(mallory.privateKeyPkcs8, wrapped)).rejects.toThrow();
  }, 30000);
});

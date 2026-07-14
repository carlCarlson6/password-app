// Tests for src/domain/crypto.ts.

import { describe, expect, it } from "vitest";

import {
  DEFAULT_KDF_PARAMS,
  decodeBase64,
  decodeBlobBase64,
  encodeBase64,
  encodeBlobBase64,
  parseBlob,
  serializeBlob,
} from "../../src/domain/crypto";

describe("KDF parameters", () => {
  it("pins the signup defaults from the security model (m=64MiB, t=3, p=4)", () => {
    // Changing these silently would change every new user's master key.
    expect(DEFAULT_KDF_PARAMS).toEqual({
      algorithm: "argon2id",
      memoryKiB: 65536,
      iterations: 3,
      parallelism: 4,
    });
  });
});

describe("base64", () => {
  it("encodes a known vector", () => {
    expect(encodeBase64(new TextEncoder().encode("hello"))).toBe("aGVsbG8=");
  });

  it("round-trips arbitrary bytes", () => {
    const bytes = new Uint8Array([0, 1, 2, 253, 254, 255]);
    expect(decodeBase64(encodeBase64(bytes))).toEqual(bytes);
  });

  it("rejects non-base64 input", () => {
    expect(() => decodeBase64("not base64!!")).toThrow(/malformed/);
  });
});

describe("cipher blob wire format", () => {
  it("round-trips through serialize/parse", () => {
    const blob = {
      iv: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
      ciphertext: new Uint8Array([42, 0, 255]),
    };
    expect(parseBlob(serializeBlob(blob))).toEqual(blob);
  });

  it("rejects an unknown version", () => {
    expect(() => parseBlob("v9.AAAA.AAAA")).toThrow(/malformed/);
  });

  it("rejects a missing segment", () => {
    expect(() => parseBlob("v1.AAAA")).toThrow(/malformed/);
  });

  it("rejects garbage base64 in a segment", () => {
    expect(() => parseBlob("v1.!!!.AAAA")).toThrow(/malformed/);
  });
});

describe("cipher blob API transport (plain base64)", () => {
  const blob = {
    iv: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
    ciphertext: new Uint8Array([42, 0, 255]),
  };

  it("produces plain base64, as the auth wire contract requires", () => {
    expect(encodeBlobBase64(blob)).toMatch(/^[A-Za-z0-9+/]+={0,2}$/);
  });

  it("round-trips and preserves the versioned blob format inside", () => {
    const encoded = encodeBlobBase64(blob);
    expect(atob(encoded)).toBe(serializeBlob(blob));
    expect(decodeBlobBase64(encoded)).toEqual(blob);
  });

  it("rejects non-base64 input", () => {
    expect(() => decodeBlobBase64("v1.AAAA.AAAA")).toThrow(/malformed/);
  });
});

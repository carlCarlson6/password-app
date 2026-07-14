import type { KdfParams } from "../../domain/crypto";

/**
 * Driven port: the four auth endpoints, exactly as the backend speaks them
 * (camelCase JSON; binary fields base64-encoded strings). Implemented in
 * `infrastructure/http/`; faked in tests.
 *
 * The wrapped-key fields are opaque strings to the server — it stores and
 * returns them without ever being able to read what is inside.
 */

export interface RegisterRequest {
  readonly email: string;
  /** base64 of the derived login credential (never the master password). */
  readonly masterPasswordHash: string;
  /** The Argon2id parameters this account's master key was derived with. */
  readonly kdf: KdfParams;
  /** USK wrapped (AES-256-GCM) under the stretched master key. */
  readonly wrappedUserSymmetricKey: string;
  /** base64 SPKI of the RSA-OAEP-2048 public key. */
  readonly publicKey: string;
  /** PKCS#8 private key wrapped (AES-256-GCM) under the USK. */
  readonly wrappedPrivateKey: string;
}

export interface LoginResponse {
  readonly accessToken: string;
  readonly wrappedUserSymmetricKey: string;
  readonly publicKey: string;
  readonly wrappedPrivateKey: string;
}

export interface AuthApi {
  /** KDF params for the email (faked by the server for unknown emails). */
  prelogin(email: string): Promise<KdfParams>;

  register(request: RegisterRequest): Promise<void>;

  /**
   * Exchanges the credential for an access token + the wrapped keys.
   * The rotating refresh token arrives as an httpOnly cookie — the
   * application layer never sees it.
   */
  login(email: string, masterPasswordHash: string): Promise<LoginResponse>;

  /** Trades the refresh cookie for a fresh access token. */
  refresh(): Promise<string>;
}

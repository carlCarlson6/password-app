import {
  DEFAULT_KDF_PARAMS,
  encodeBase64,
  encodeBlobBase64,
  type KdfParams,
} from "../domain/crypto";
import type { AuthApi, CryptoService } from "./ports";

/**
 * Signup: derive and wrap every key client-side, then ship only wrapped
 * material + the login hash. The master password, the Master Key and the
 * unwrapped User Symmetric Key / private key never leave this function.
 *
 * `kdf` is injectable so tests can use cheap Argon2id costs; production
 * wiring uses the pinned signup defaults.
 */
export function makeRegisterUser(
  crypto: CryptoService,
  authApi: AuthApi,
  kdf: KdfParams = DEFAULT_KDF_PARAMS,
) {
  return async function registerUser(
    email: string,
    masterPassword: string,
  ): Promise<void> {
    // Master Key: Argon2id(password, salt=email) — deterministic per account.
    const masterKey = await crypto.deriveMasterKey(masterPassword, email, kdf);

    // Two independent derivations of the Master Key: the login credential
    // (sent to the server) and the key-wrapping key (never sent).
    const [masterPasswordHash, stretchedMasterKey] = await Promise.all([
      crypto.deriveMasterPasswordHash(masterKey, masterPassword),
      crypto.stretchMasterKey(masterKey),
    ]);

    // The User Symmetric Key is random — changing the master password later
    // only re-wraps it, never re-encrypts data.
    const userSymmetricKey = crypto.generateSymmetricKey();
    const wrappedUserSymmetricKey = await crypto.encrypt(
      stretchedMasterKey,
      userSymmetricKey,
    );

    // Keypair at signup so sharing (v2) needs no migration. Only the private
    // key is secret; it travels wrapped under the USK.
    const keyPair = await crypto.generateRsaKeyPair();
    const wrappedPrivateKey = await crypto.encrypt(
      userSymmetricKey,
      keyPair.privateKeyPkcs8,
    );

    await authApi.register({
      email,
      masterPasswordHash: encodeBase64(masterPasswordHash),
      kdf,
      wrappedUserSymmetricKey: encodeBlobBase64(wrappedUserSymmetricKey),
      publicKey: encodeBase64(keyPair.publicKeySpki),
      wrappedPrivateKey: encodeBlobBase64(wrappedPrivateKey),
    });
  };
}

export type RegisterUser = ReturnType<typeof makeRegisterUser>;

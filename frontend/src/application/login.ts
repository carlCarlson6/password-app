import {
  decodeBase64,
  decodeBlobBase64,
  encodeBase64,
  type SymmetricKey,
} from "../domain/crypto";
import type { AuthApi, CryptoService, KeyStore } from "./ports";

/**
 * Login: prelogin for the account's KDF params, re-derive the Master Key and
 * the login credential, exchange it for the wrapped keys, and unwrap them
 * locally into the KeyStore. Nothing secret is sent; nothing unwrapped is
 * persisted — the unlocked keys live only in the (in-memory) KeyStore.
 *
 * A wrong master password surfaces either as a rejected login (server-side
 * hash mismatch) or as a failed AES-GCM unwrap — both reject this promise.
 */
export function makeLogin(
  crypto: CryptoService,
  authApi: AuthApi,
  keyStore: KeyStore,
) {
  return async function login(email: string, masterPassword: string): Promise<void> {
    const kdf = await authApi.prelogin(email);

    const masterKey = await crypto.deriveMasterKey(masterPassword, email, kdf);
    const masterPasswordHash = await crypto.deriveMasterPasswordHash(
      masterKey,
      masterPassword,
    );

    const response = await authApi.login(email, encodeBase64(masterPasswordHash));

    // Unwrap locally: stretched master key → USK → private key. The GCM tag
    // check makes a wrong key or tampered blob throw instead of yielding junk.
    const stretchedMasterKey = await crypto.stretchMasterKey(masterKey);
    const userSymmetricKey = (await crypto.decrypt(
      stretchedMasterKey,
      decodeBlobBase64(response.wrappedUserSymmetricKey),
    )) as SymmetricKey;
    const privateKeyPkcs8 = await crypto.decrypt(
      userSymmetricKey,
      decodeBlobBase64(response.wrappedPrivateKey),
    );

    keyStore.set({
      userSymmetricKey,
      publicKeySpki: decodeBase64(response.publicKey),
      privateKeyPkcs8,
    });
  };
}

export type Login = ReturnType<typeof makeLogin>;

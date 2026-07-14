import type { EncryptedBlob, KdfParams, SymmetricKey } from "../domain/crypto";
import type { CryptoService } from "./ports";
import type { KeyStore } from "./ports/keyStore";

/**
 * Everything needed to re-unlock besides the master password. All of it is
 * either public (email, KDF params) or ciphertext (the wrapped keys, as
 * returned by login), so it is safe to retain in memory across a lock.
 */
export interface UnlockContext {
  readonly email: string;
  readonly kdfParams: KdfParams;
  /** User Symmetric Key, AES-256-GCM-wrapped under the Stretched Master Key. */
  readonly wrappedUserSymmetricKey: EncryptedBlob;
  /** RSA public key (SPKI bytes) — public material, retained so a re-unlock
   *  restores exactly the `UnlockedKeys` that login produced. */
  readonly publicKeySpki: Uint8Array;
  /** RSA private key (PKCS#8), AES-256-GCM-wrapped under the User Symmetric Key. */
  readonly wrappedPrivateKey: EncryptedBlob;
}

/**
 * Re-derive the key hierarchy from the master password and repopulate the
 * key store. Returns `true` on success, `false` on a wrong password (AES-GCM
 * authentication failure while unwrapping) — in that case the store is left
 * locked and untouched.
 */
export function makeUnlockVault(cryptoService: CryptoService, keyStore: KeyStore) {
  return async function unlockVault(
    context: UnlockContext,
    masterPassword: string,
  ): Promise<boolean> {
    const masterKey = await cryptoService.deriveMasterKey(
      masterPassword,
      context.email,
      context.kdfParams,
    );
    const stretchedKey = await cryptoService.stretchMasterKey(masterKey);

    try {
      const userSymmetricKey = (await cryptoService.decrypt(
        stretchedKey,
        context.wrappedUserSymmetricKey,
      )) as SymmetricKey; // unwrapping restores the branded key it once was
      const privateKeyPkcs8 = await cryptoService.decrypt(
        userSymmetricKey,
        context.wrappedPrivateKey,
      );
      keyStore.set({
        userSymmetricKey,
        publicKeySpki: context.publicKeySpki,
        privateKeyPkcs8,
      });
      return true;
    } catch {
      // Wrong master password: GCM's auth tag check rejects the unwrap.
      return false;
    } finally {
      // The intermediate keys served their purpose — zeroize best-effort.
      masterKey.fill(0);
      stretchedKey.fill(0);
    }
  };
}

export type UnlockVault = ReturnType<typeof makeUnlockVault>;

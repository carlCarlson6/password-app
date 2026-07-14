use crate::identity::{CredentialHash, EmailAddress, KdfParams, KeyBlob, UserId};

/// The Identity aggregate root: everything the server knows about one user.
///
/// Note what is ABSENT: no master password, no unwrapped key, nothing
/// decryptable. The three [`KeyBlob`]s are ciphertext the client produced;
/// the credential is stored only as its server-side re-hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAccount {
    id: UserId,
    email: EmailAddress,
    credential_hash: CredentialHash,
    kdf: KdfParams,
    wrapped_user_symmetric_key: KeyBlob,
    public_key: KeyBlob,
    wrapped_private_key: KeyBlob,
}

impl UserAccount {
    /// Assemble an account from already-validated value objects. Every field
    /// carries its own invariants, so construction cannot fail — invalid
    /// states were rejected upstream, at each VO's `new`.
    //
    // Rust note: parameters are taken BY VALUE (no `&`), so the aggregate
    // owns its parts outright — no lifetimes to manage, and callers must
    // `.clone()` explicitly if they want to keep a copy.
    pub fn new(
        id: UserId,
        email: EmailAddress,
        credential_hash: CredentialHash,
        kdf: KdfParams,
        wrapped_user_symmetric_key: KeyBlob,
        public_key: KeyBlob,
        wrapped_private_key: KeyBlob,
    ) -> Self {
        Self {
            id,
            email,
            credential_hash,
            kdf,
            wrapped_user_symmetric_key,
            public_key,
            wrapped_private_key,
        }
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    pub fn credential_hash(&self) -> &CredentialHash {
        &self.credential_hash
    }

    pub fn kdf(&self) -> KdfParams {
        self.kdf
    }

    pub fn wrapped_user_symmetric_key(&self) -> &KeyBlob {
        &self.wrapped_user_symmetric_key
    }

    pub fn public_key(&self) -> &KeyBlob {
        &self.public_key
    }

    pub fn wrapped_private_key(&self) -> &KeyBlob {
        &self.wrapped_private_key
    }
}

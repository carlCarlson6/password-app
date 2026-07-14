use async_trait::async_trait;

use domain::identity::{CredentialHash, MasterPasswordHash};

/// Driven port: server-side Argon2id re-hash of the client's login
/// credential. Async because a good password hash is deliberately slow —
/// adapters run it off the async runtime's worker threads.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Re-hash the client-supplied credential for at-rest storage.
    async fn hash(
        &self,
        credential: &MasterPasswordHash,
    ) -> Result<CredentialHash, PasswordHasherError>;

    /// Verify a presented credential.
    ///
    /// `stored = None` means "no such user": the adapter MUST still burn a
    /// full verification against a dummy hash and return `Ok(false)`, so an
    /// unknown email costs the same wall-clock time as a wrong password —
    /// the anti-enumeration contract lives in this signature.
    async fn verify(
        &self,
        candidate: &MasterPasswordHash,
        stored: Option<&CredentialHash>,
    ) -> Result<bool, PasswordHasherError>;
}

#[derive(Debug, thiserror::Error)]
#[error("password hashing failure: {0}")]
pub struct PasswordHasherError(pub String);

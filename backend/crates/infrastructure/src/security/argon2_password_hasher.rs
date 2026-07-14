use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString};
use async_trait::async_trait;

use application::ports::{PasswordHasher, PasswordHasherError};
use domain::identity::{CredentialHash, MasterPasswordHash};

/// Argon2id adapter for the [`PasswordHasher`] port: the at-rest RE-hash of
/// the client's (already Argon2id-derived) login credential.
///
/// Uses the `argon2` crate defaults (Argon2id v19, m=19456 KiB, t=2, p=1) —
/// sized for server throughput; the expensive client-side hash already
/// happened. The salt is random per hash, so equal credentials never share
/// a stored hash.
pub struct Argon2PasswordHasher {
    /// A real hash of a fixed dummy credential. Verifying against it when no
    /// user exists burns the same CPU as a genuine check, closing the
    /// "unknown email returns instantly" timing oracle.
    dummy_hash: String,
}

impl Argon2PasswordHasher {
    /// Computes the dummy hash once; call at startup (it costs one Argon2).
    pub fn new() -> Result<Self, PasswordHasherError> {
        let dummy_hash = hash_blocking(b"decoy-credential-for-unknown-emails".to_vec())?;
        Ok(Self { dummy_hash })
    }
}

fn internal(reason: impl ToString) -> PasswordHasherError {
    PasswordHasherError(reason.to_string())
}

// Rust note: free functions taking OWNED Vec<u8> so they can move into
// `spawn_blocking` closures, which must be `'static` (no borrows of &self).
fn hash_blocking(credential: Vec<u8>) -> Result<String, PasswordHasherError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(&credential, &salt)
        .map(|hash| hash.to_string())
        .map_err(internal)
}

fn verify_blocking(candidate: Vec<u8>, phc: String) -> Result<bool, PasswordHasherError> {
    let parsed = PasswordHash::new(&phc).map_err(internal)?;
    // Mismatch is a normal `false`, not an error; anything else bubbles up.
    match Argon2::default().verify_password(&candidate, &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(error) => Err(internal(error)),
    }
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(
        &self,
        credential: &MasterPasswordHash,
    ) -> Result<CredentialHash, PasswordHasherError> {
        let bytes = credential.as_bytes().to_vec();
        // Rust note: `spawn_blocking` ships the closure to a thread pool for
        // blocking work, so a slow Argon2 never stalls the async executor.
        // The outer `?` is the join error, the inner the hashing error.
        let phc = tokio::task::spawn_blocking(move || hash_blocking(bytes))
            .await
            .map_err(internal)??;
        CredentialHash::new(phc).map_err(internal)
    }

    async fn verify(
        &self,
        candidate: &MasterPasswordHash,
        stored: Option<&CredentialHash>,
    ) -> Result<bool, PasswordHasherError> {
        let is_dummy = stored.is_none();
        let phc = stored
            .map(|hash| hash.as_str().to_string())
            .unwrap_or_else(|| self.dummy_hash.clone());
        let bytes = candidate.as_bytes().to_vec();

        let matched = tokio::task::spawn_blocking(move || verify_blocking(bytes, phc))
            .await
            .map_err(internal)??;

        // The dummy path NEVER authenticates, even if someone guessed the
        // decoy credential.
        Ok(matched && !is_dummy)
    }
}

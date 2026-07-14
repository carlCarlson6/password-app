use std::sync::Arc;

use domain::identity::{
    EmailAddress, KdfAlgorithm, KdfParams, KeyBlob, MasterPasswordHash, UserAccount, UserId,
};
use domain::shared::DomainError;

use crate::ports::{
    IdGenerator, PasswordHasher, PasswordHasherError, UserRepository, UserRepositoryError,
};

/// Raw registration input, base64 already decoded at the API edge.
/// Plain bytes/strings in — the use case turns them into value objects and
/// rejects anything invalid before touching a port.
pub struct RegisterUserInput {
    pub email: String,
    pub master_password_hash: Vec<u8>,
    pub kdf_algorithm: String,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub wrapped_user_symmetric_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub wrapped_private_key: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterUserError {
    /// Malformed input (bad email, out-of-range KDF params, …) → HTTP 400.
    #[error(transparent)]
    Invalid(#[from] DomainError),

    /// Infrastructure trouble → HTTP 500. Duplicate email is NOT here: it is
    /// deliberately reported as success (anti-enumeration, see README).
    #[error("registration failed: {0}")]
    Infra(String),
}

/// Use case: create an account from client-derived key material.
///
/// The server re-hashes the login credential and stores the wrapped keys as
/// opaque bytes — nothing here can decrypt anything.
pub struct RegisterUser {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    ids: Arc<dyn IdGenerator>,
}

impl RegisterUser {
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { users, hasher, ids }
    }

    pub async fn execute(&self, input: RegisterUserInput) -> Result<(), RegisterUserError> {
        // Rust note: `?` on a Result whose error implements `From<DomainError>`
        // auto-converts into RegisterUserError::Invalid (the `#[from]` above).
        let email = EmailAddress::new(input.email)?;
        let credential = MasterPasswordHash::new(input.master_password_hash)?;
        let kdf = KdfParams::new(
            KdfAlgorithm::parse(&input.kdf_algorithm)?,
            input.kdf_memory_kib,
            input.kdf_iterations,
            input.kdf_parallelism,
        )?;
        let wrapped_usk = KeyBlob::new(input.wrapped_user_symmetric_key)?;
        let public_key = KeyBlob::new(input.public_key)?;
        let wrapped_private_key = KeyBlob::new(input.wrapped_private_key)?;

        // Hash BEFORE the duplicate check so a duplicate registration costs
        // the same time as a fresh one — no timing oracle on email existence.
        let credential_hash = self.hasher.hash(&credential).await?;

        let account = UserAccount::new(
            UserId::new(self.ids.generate())?,
            email,
            credential_hash,
            kdf,
            wrapped_usk,
            public_key,
            wrapped_private_key,
        );

        match self.users.insert(&account).await {
            Ok(()) => Ok(()),
            // Anti-enumeration decision (documented in README): a duplicate
            // email is silently reported as success. The real owner already
            // has their account; an attacker learns nothing.
            Err(UserRepositoryError::DuplicateEmail) => Ok(()),
            Err(UserRepositoryError::Store(reason)) => Err(RegisterUserError::Infra(reason)),
        }
    }
}

// Rust note: manual `From` impls (instead of `#[from]`) because two source
// errors collapse into the same `Infra` variant — `#[from]` allows only one.
impl From<PasswordHasherError> for RegisterUserError {
    fn from(error: PasswordHasherError) -> Self {
        Self::Infra(error.to_string())
    }
}

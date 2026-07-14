use crate::shared::DomainError;

/// The server-side Argon2id re-hash of the client's login credential, in
/// PHC string format (`$argon2id$v=19$m=...$salt$hash`).
///
/// This — never the raw [`MasterPasswordHash`] — is what the `users` table
/// stores. The domain treats it as opaque; only the `PasswordHasher`
/// adapter can produce or verify one.
///
/// [`MasterPasswordHash`]: crate::identity::MasterPasswordHash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHash(String);

impl CredentialHash {
    pub fn new(phc: impl Into<String>) -> Result<Self, DomainError> {
        let value = phc.into();
        // Shallow check: a PHC string always starts with '$'. Full parsing
        // belongs to the hashing adapter, not the domain.
        if value.is_empty() || !value.starts_with('$') {
            return Err(DomainError::InvalidValue {
                field: "credential hash",
                reason: "not a PHC-format hash string",
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

use crate::shared::DomainError;

/// Opaque wrapped-key material (wrapped User Symmetric Key, public key,
/// wrapped private key). The server stores these bytes and NEVER interprets
/// them — zero-knowledge means the domain model cannot even name what is
/// inside. Base64 encoding/decoding happens at the API edge.
//
// Rust note: `Vec<u8>` owns its bytes on the heap. Deriving `PartialEq`
// compares contents (value-object semantics), not pointers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBlob(Vec<u8>);

impl KeyBlob {
    /// Generous ceiling: the largest real blob (encrypted RSA-2048 PKCS#8)
    /// is ~2 KiB; anything near the cap is abuse, not key material.
    pub const MAX_LEN: usize = 8 * 1024;

    pub fn new(bytes: Vec<u8>) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::InvalidValue {
            field: "key blob",
            reason,
        };
        if bytes.is_empty() {
            return Err(invalid("must not be empty"));
        }
        if bytes.len() > Self::MAX_LEN {
            return Err(invalid("larger than 8 KiB"));
        }
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The login credential as the client sent it: Argon2id(Master Key,
/// master password), base64-decoded at the API edge. Held only transiently —
/// the server re-hashes it (see [`CredentialHash`]) and never persists or
/// logs the raw value.
///
/// [`CredentialHash`]: crate::identity::CredentialHash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasterPasswordHash(Vec<u8>);

impl MasterPasswordHash {
    pub const MIN_LEN: usize = 16;
    pub const MAX_LEN: usize = 128;

    pub fn new(bytes: Vec<u8>) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::InvalidValue {
            field: "master password hash",
            reason,
        };
        if !(Self::MIN_LEN..=Self::MAX_LEN).contains(&bytes.len()) {
            return Err(invalid("length outside 16..=128 bytes"));
        }
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

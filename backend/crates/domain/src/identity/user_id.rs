use std::fmt;

use crate::shared::DomainError;

/// Opaque identifier of a [`UserAccount`](crate::identity::UserAccount).
///
/// The domain does not mint ids (that would require randomness — I/O by
/// spirit); the application layer generates them through an `IdGenerator`
/// port and the domain only enforces that an id is never blank.
//
// Rust note: same "newtype" pattern as `EmailAddress` — the private inner
// String makes `UserId::new` the only door, so invariants hold everywhere.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let value = raw.into();
        if value.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "user id",
                reason: "must not be blank",
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

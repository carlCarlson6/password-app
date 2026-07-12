use std::fmt;

use crate::shared::DomainError;

/// A normalized (trimmed, lowercased) email address.
///
/// Value object: equality by value, immutable after construction.
/// Validation is intentionally shallow — real ownership is proven by
/// verification mail (Phase 4), not by grammar.
//
// Rust note: this is the "newtype" pattern — a tuple struct wrapping one
// field. The inner `String` is private, so the ONLY way to obtain an
// `EmailAddress` is through `new()`, which enforces the invariants. Invalid
// states are unrepresentable. `Clone` allows explicit copies, `Hash` lets it
// be a HashMap key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    // Rust note: `impl Into<String>` accepts anything convertible into a
    // String (&str, String, ...). Returning `Result<Self, DomainError>` makes
    // failure explicit — Rust has no exceptions; callers must handle the `Err`
    // case or pass it up.
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let value = raw.into().trim().to_lowercase();

        // Rust note: a closure (anonymous function) to avoid repeating the
        // error construction below. `|reason|` is the parameter list.
        let invalid = |reason| DomainError::InvalidValue {
            field: "email",
            reason,
        };

        if value.len() > 254 {
            return Err(invalid("longer than 254 characters"));
        }
        // Rust note: `let ... else` destructures if the pattern matches and
        // diverges (returns) otherwise. `split_once` returns an
        // `Option<(&str, &str)>` — `Some((local, host))` binds both halves.
        let Some((local, host)) = value.split_once('@') else {
            return Err(invalid("missing '@'"));
        };
        if local.is_empty() {
            return Err(invalid("empty local part"));
        }
        if !host.contains('.') || host.starts_with('.') || host.ends_with('.') {
            return Err(invalid("malformed domain"));
        }

        Ok(Self(value))
    }

    // Rust note: `&self` borrows the value immutably (no ownership transfer);
    // `&str` borrows the inner String's bytes. Zero copies happen here.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Rust note: implementing the standard `Display` trait gives `{}` formatting
// and a free `.to_string()` method.
impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

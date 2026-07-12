//! Identity context: user accounts, KDF parameters, wrapped user keys.
//!
//! Fleshed out in Phase 1. For now it holds [`EmailAddress`], the value
//! object used as the account identifier and KDF salt input.

mod email_address;

pub use email_address::EmailAddress;

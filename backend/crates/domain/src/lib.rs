//! Pure domain layer: entities, value objects, domain errors.
//!
//! Dependency rule: this crate depends on nothing but the standard library
//! (plus `thiserror` for error ergonomics). No async, no I/O, no frameworks.
//! If it can't be exercised from a plain unit test, it doesn't belong here.
//!
//! Bounded contexts (see README §3):
//! - [`identity`]: user accounts, login credential verification, wrapped keys
//! - [`vaulting`]: vault and item ciphertext lifecycle
//! - [`access`]: vault grants — who holds a wrapped copy of which vault key

// Rust note: `//!` comments document the enclosing item (here: the whole crate);
// `///` comments document the item that follows them. Both show up in `cargo doc`.

// Rust note: Rust's module tree is explicit. `pub mod access;` declares a public
// module and tells the compiler to load it from `access/mod.rs` (or `access.rs`).
// Nothing is importable unless some `mod` declaration reaches it from lib.rs.
pub mod access;
pub mod identity;
pub mod shared;
pub mod vaulting;

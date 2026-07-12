//! Vaulting context: vault lifecycle and item ciphertext CRUD.
//!
//! Fleshed out in Phase 2. Server-side, an item's content is an opaque
//! `CipherBlob` — this context never models usernames, passwords, or any
//! decrypted structure.

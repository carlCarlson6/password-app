//! Test harness for the `infrastructure` crate.
//!
//! Project convention: all tests live under `tests/`, mirroring the `src/`
//! module tree — never inline `#[cfg(test)]` modules in source files.

mod persistence;
mod security;

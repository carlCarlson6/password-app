//! Test harness for the `api` crate.
//!
//! Project convention: all tests live under `tests/`, mirroring the `src/`
//! module tree — never inline `#[cfg(test)]` modules in source files.

mod handlers;
mod middleware;
mod support;

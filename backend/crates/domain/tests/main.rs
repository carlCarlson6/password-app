//! Test harness for the `domain` crate.
//!
//! Project convention: all tests live under `tests/`, mirroring the `src/`
//! module tree — never inline `#[cfg(test)]` modules in source files.
//
// Rust note: Cargo compiles each TOP-LEVEL `.rs` file in `tests/` as its own
// test crate linked against the library's public API. Files in subdirectories
// are ignored unless a `mod` declaration reaches them from here — that's what
// lets us mirror the `src/` folder structure inside one test binary.

mod identity;

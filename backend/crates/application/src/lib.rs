//! Application layer: use cases (driving side) and port traits (driven side).
//!
//! Dependency rule: depends on `domain` only. Infrastructure implements the
//! traits in [`ports`]; the `api` crate calls the use cases in [`use_cases`].
//! A use case is the transaction boundary and holds no framework types.

pub mod ports;
pub mod use_cases;

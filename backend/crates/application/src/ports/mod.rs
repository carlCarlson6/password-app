//! Driven ports: interfaces the application core needs the outside world to
//! implement (persistence, crypto, clocks, …). Adapters live in the
//! `infrastructure` crate.

mod database_probe;

pub use database_probe::{DatabaseProbe, ProbeError};

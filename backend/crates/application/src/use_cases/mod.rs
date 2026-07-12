//! Use cases: one module per business operation. HTTP handlers call these
//! and nothing else.

pub mod check_health;

pub use check_health::{CheckHealth, ComponentStatus, HealthReport};

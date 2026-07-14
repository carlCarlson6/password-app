//! Driving adapter: Axum HTTP layer.
//!
//! Handlers stay thin — deserialize, call a use case, map to a response DTO.
//! No handler talks to SQLx or contains business rules.
//
// Rust note: this crate has BOTH a lib.rs and a main.rs. The binary target
// (main.rs) is just the composition root; everything testable lives in the
// library so the `tests/` harness can build a router without spawning a server.

pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

pub use routes::{build_router, build_router_with};
pub use state::AppState;

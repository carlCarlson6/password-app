use std::sync::Arc;

use application::use_cases::CheckHealth;

/// Shared handler dependencies: one field per use case.
///
/// Built once in the composition root (`main.rs`, or a test), then cloned
/// cheaply into every request handler by Axum.
//
// Rust note: `Arc<T>` is an atomically reference-counted pointer — cloning the
// state clones pointers, not the use cases themselves. Axum requires state to
// be `Clone` because each request gets its own copy.
#[derive(Clone)]
pub struct AppState {
    pub check_health: Arc<CheckHealth>,
}

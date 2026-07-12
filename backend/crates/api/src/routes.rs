use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::handlers::health;
use crate::state::AppState;

/// Assemble the full HTTP surface. The only place routes are declared.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::get_health))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

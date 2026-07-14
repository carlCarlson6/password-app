use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use crate::handlers::{auth, health};
use crate::middleware::{RateLimitConfig, RateLimiter, rate_limit};
use crate::state::AppState;

/// Assemble the full HTTP surface with production rate limits.
pub fn build_router(state: AppState) -> Router {
    build_router_with(state, RateLimitConfig::default())
}

/// Same, with injectable rate limits — the only place routes are declared.
pub fn build_router_with(state: AppState, rate_limit_config: RateLimitConfig) -> Router {
    let limiter = RateLimiter::new(rate_limit_config);

    // The auth group carries the rate limiter; hammering login/refresh gets
    // exponentially slower (429 + Retry-After) instead of free brute force.
    let auth_routes = Router::new()
        .route("/prelogin", post(auth::prelogin))
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .layer(axum::middleware::from_fn_with_state(limiter, rate_limit));

    Router::new()
        .route("/api/health", get(health::get_health))
        .nest("/api/auth", auth_routes)
        // NOTE: TraceLayer logs method/path/status only — request bodies on
        // auth routes (credential hashes, wrapped keys) are never logged.
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

//! Tests for `api/src/middleware/rate_limit.rs`.
//!
//! Uses a real router with stub-backed state (no database needed — the
//! limiter rejects before any handler runs) and tiny injected limits.
//! Without ConnectInfo (tower::oneshot), all requests share one bucket,
//! which is exactly what these tests need.

use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

use api::build_router_with;
use api::middleware::RateLimitConfig;
use application::ports::{DatabaseProbe, ProbeError};

use crate::support::state_with_probe;

struct StubProbe;
#[async_trait]
impl DatabaseProbe for StubProbe {
    async fn ping(&self) -> Result<(), ProbeError> {
        Ok(())
    }
}

fn app(config: RateLimitConfig) -> Router {
    build_router_with(state_with_probe(StubProbe), config)
}

fn tight_config() -> RateLimitConfig {
    RateLimitConfig {
        max_requests: 2,
        window: Duration::from_secs(60),
        base_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(4),
    }
}

async fn hit(app: &Router, path: &str) -> (StatusCode, Option<String>) {
    let response = app
        .clone()
        .oneshot(
            Request::post(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"email":"a@example.com"}"#))
                .expect("request"),
        )
        .await
        .expect("infallible router");
    let retry_after = response
        .headers()
        .get(header::RETRY_AFTER)
        .map(|v| v.to_str().expect("ascii").to_string());
    (response.status(), retry_after)
}

#[tokio::test]
async fn requests_beyond_the_window_budget_get_429_with_retry_after() {
    let app = app(tight_config());

    // Two within budget…
    for _ in 0..2 {
        let (status, _) = hit(&app, "/api/auth/prelogin").await;
        assert_eq!(status, StatusCode::OK);
    }
    // …the third trips the limiter.
    let (status, retry_after) = hit(&app, "/api/auth/prelogin").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(retry_after.as_deref(), Some("1"));
}

#[tokio::test]
async fn hammering_while_blocked_backs_off_exponentially_up_to_the_cap() {
    let app = app(tight_config());
    for _ in 0..2 {
        hit(&app, "/api/auth/prelogin").await;
    }

    // Violation, then three more attempts while blocked: 1s, 2s, 4s, 4s (cap).
    let mut retry_afters = Vec::new();
    for _ in 0..4 {
        let (status, retry_after) = hit(&app, "/api/auth/prelogin").await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        retry_afters.push(retry_after.expect("Retry-After header"));
    }
    assert_eq!(retry_afters, ["1", "2", "4", "4"]);
}

#[tokio::test]
async fn the_limit_covers_every_auth_route_but_not_health() {
    let app = app(RateLimitConfig {
        max_requests: 1,
        ..tight_config()
    });

    let (first, _) = hit(&app, "/api/auth/login").await;
    assert_ne!(first, StatusCode::TOO_MANY_REQUESTS); // budget spent here

    // One shared bucket (no ConnectInfo in oneshot): every auth route blocks.
    for path in [
        "/api/auth/prelogin",
        "/api/auth/register",
        "/api/auth/login",
        "/api/auth/refresh",
    ] {
        let (status, _) = hit(&app, path).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "path {path}");
    }

    // Health sits outside the limited group.
    let health = app
        .clone()
        .oneshot(
            Request::get("/api/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("infallible router");
    assert_eq!(health.status(), StatusCode::OK);
}

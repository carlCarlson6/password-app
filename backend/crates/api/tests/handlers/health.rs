//! Tests for `api/src/handlers/health.rs`.
//!
//! The "up" test is the Phase 0 walking skeleton: a real request through the
//! router → use case → SQLite adapter and back, no HTTP socket needed.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use api::{AppState, build_router};
use application::ports::{DatabaseProbe, ProbeError};
use application::use_cases::CheckHealth;
use infrastructure::persistence::{SqliteDatabaseProbe, connect, run_migrations};

// Rust note: `impl Trait` in argument position — accepts any concrete probe
// type; the function body decides nothing about which adapter it gets.
fn app_with(probe: impl DatabaseProbe + 'static) -> axum::Router {
    build_router(AppState {
        check_health: Arc::new(CheckHealth::new(Arc::new(probe))),
    })
}

async fn get_health(app: axum::Router) -> (StatusCode, serde_json::Value) {
    // Rust note: `oneshot` (from tower's ServiceExt) drives one request
    // through the whole middleware/router stack in-process.
    let response = app
        .oneshot(
            Request::get("/api/health")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("infallible router");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body read")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).expect("json body");
    (status, json)
}

#[tokio::test]
async fn returns_ok_with_a_reachable_database() {
    let pool = connect("sqlite::memory:").await.expect("connect");
    run_migrations(&pool).await.expect("migrate");

    let (status, body) = get_health(app_with(SqliteDatabaseProbe::new(pool))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], "up");
}

struct FailingProbe;

#[async_trait]
impl DatabaseProbe for FailingProbe {
    async fn ping(&self) -> Result<(), ProbeError> {
        Err(ProbeError {
            reason: "unreachable".into(),
        })
    }
}

#[tokio::test]
async fn returns_service_unavailable_when_the_database_is_down() {
    let (status, body) = get_health(app_with(FailingProbe)).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["database"], "down");
}

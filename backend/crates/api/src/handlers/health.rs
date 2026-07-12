use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use application::use_cases::{ComponentStatus, HealthReport};

use crate::state::AppState;

/// Response DTO — the wire shape is owned by the api crate, never by the
/// application layer.
#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

// Rust note: `From` is the standard conversion trait; implementing it gives
// `HealthResponse::from(report)` and `report.into()`. DTO mapping lives here,
// at the edge, so the use case stays transport-agnostic.
impl From<HealthReport> for HealthResponse {
    fn from(report: HealthReport) -> Self {
        let label = |status: ComponentStatus| match status {
            ComponentStatus::Up => "up",
            ComponentStatus::Down => "down",
        };
        Self {
            status: if report.is_healthy() {
                "ok"
            } else {
                "degraded"
            },
            database: label(report.database),
        }
    }
}

/// GET /api/health — the walking skeleton's end-to-end request.
//
// Rust note: Axum extractors are just function parameters: `State(state)`
// pattern-matches the shared state out of its wrapper. The return tuple
// (status code, JSON body) already implements `IntoResponse`.
pub async fn get_health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let report = state.check_health.execute().await;
    let code = if report.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(report.into()))
}

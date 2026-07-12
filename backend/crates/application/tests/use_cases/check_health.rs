//! Tests for `application/src/use_cases/check_health.rs`.

use std::sync::Arc;

use async_trait::async_trait;

use application::ports::{DatabaseProbe, ProbeError};
use application::use_cases::{CheckHealth, ComponentStatus};

// Rust note: tests in `tests/` are a separate crate, so they can only touch
// the library's PUBLIC api — a stub implements the public port trait here.
struct StubProbe {
    healthy: bool,
}

#[async_trait]
impl DatabaseProbe for StubProbe {
    async fn ping(&self) -> Result<(), ProbeError> {
        if self.healthy {
            Ok(())
        } else {
            Err(ProbeError {
                reason: "stub failure".into(),
            })
        }
    }
}

#[tokio::test]
async fn reports_up_when_probe_succeeds() {
    let report = CheckHealth::new(Arc::new(StubProbe { healthy: true }))
        .execute()
        .await;
    assert_eq!(report.database, ComponentStatus::Up);
    assert!(report.is_healthy());
}

#[tokio::test]
async fn reports_down_when_probe_fails() {
    let report = CheckHealth::new(Arc::new(StubProbe { healthy: false }))
        .execute()
        .await;
    assert_eq!(report.database, ComponentStatus::Down);
    assert!(!report.is_healthy());
}

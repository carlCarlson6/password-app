use std::sync::Arc;

use crate::ports::DatabaseProbe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct HealthReport {
    pub database: ComponentStatus,
}

impl HealthReport {
    pub fn is_healthy(&self) -> bool {
        self.database == ComponentStatus::Up
    }
}

/// Walking-skeleton use case: report whether the app's dependencies are
/// reachable. Exercises the full port/adapter wiring end to end.
pub struct CheckHealth {
    probe: Arc<dyn DatabaseProbe>,
}

impl CheckHealth {
    pub fn new(probe: Arc<dyn DatabaseProbe>) -> Self {
        Self { probe }
    }

    pub async fn execute(&self) -> HealthReport {
        let database = match self.probe.ping().await {
            Ok(()) => ComponentStatus::Up,
            Err(_) => ComponentStatus::Down,
        };
        HealthReport { database }
    }
}

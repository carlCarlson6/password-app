use std::time::{SystemTime, UNIX_EPOCH};

use application::ports::Clock;

/// Wall-clock adapter for the [`Clock`] port.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        // Rust note: `duration_since` fails only if the system clock sits
        // before 1970; falling back to 0 beats panicking in production.
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

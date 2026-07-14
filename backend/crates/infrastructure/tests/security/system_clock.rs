//! Tests for `infrastructure/src/security/system_clock.rs`.

use application::ports::Clock;
use infrastructure::security::SystemClock;

#[test]
fn reports_a_plausible_present_time() {
    let now = SystemClock.now_unix();
    // Between 2026-01-01 and 2100-01-01 — catches unit mix-ups (ms vs s).
    assert!(now > 1_767_225_600, "clock reports the past: {now}");
    assert!(now < 4_102_444_800, "clock reports the far future: {now}");
}

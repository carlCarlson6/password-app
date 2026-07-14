use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Tunables for the auth-route rate limiter. Injectable so tests can use
/// tiny numbers; production uses `default()`.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Requests allowed per client per window.
    pub max_requests: u32,
    pub window: Duration,
    /// First block length; doubles with every further violation.
    pub base_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 20,
            window: Duration::from_secs(60),
            base_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(15 * 60),
        }
    }
}

struct Visitor {
    window_start: Instant,
    count: u32,
    /// Consecutive violations; drives the exponential backoff.
    strikes: u32,
    blocked_until: Option<Instant>,
}

/// Per-client sliding-window limiter with exponential backoff:
/// exceed the window budget and you are blocked for `base_backoff`;
/// keep hammering while blocked and every attempt DOUBLES the block
/// (capped). Behaving for a full window decays one strike.
///
/// State is in-process (fine for a single node); eviction of idle entries
/// and cross-node limits are Phase 4 concerns.
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    // Rust note: `Arc<Mutex<..>>` = shared ownership across cloned router
    // layers + exclusive access for each check. Contention is trivial here
    // (auth routes), so a Mutex beats fancier structures.
    visitors: Arc<Mutex<HashMap<String, Visitor>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            visitors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn backoff_for(&self, strikes: u32) -> Duration {
        // Rust note: `<<` is a bit shift: 1 << n == 2^n. Clamped so the
        // multiplication can never overflow `Duration`.
        let factor = 1u32 << strikes.saturating_sub(1).min(20);
        (self.config.base_backoff * factor).min(self.config.max_backoff)
    }

    /// `Ok(())` = pass; `Err(d)` = blocked, retry after `d`.
    fn check(&self, key: &str, now: Instant) -> Result<(), Duration> {
        let mut visitors = self.visitors.lock().expect("rate limiter lock");
        let visitor = visitors.entry(key.to_string()).or_insert(Visitor {
            window_start: now,
            count: 0,
            strikes: 0,
            blocked_until: None,
        });

        if let Some(until) = visitor.blocked_until {
            if now < until {
                // Hammering while blocked escalates the block exponentially.
                visitor.strikes = visitor.strikes.saturating_add(1);
                let backoff = self.backoff_for(visitor.strikes);
                visitor.blocked_until = Some(now + backoff);
                return Err(backoff);
            }
            // Block served; start a fresh window (strikes are remembered).
            visitor.blocked_until = None;
            visitor.window_start = now;
            visitor.count = 0;
        }

        if now.duration_since(visitor.window_start) >= self.config.window {
            visitor.window_start = now;
            visitor.count = 0;
            visitor.strikes = visitor.strikes.saturating_sub(1); // decay
        }

        visitor.count += 1;
        if visitor.count > self.config.max_requests {
            visitor.strikes = visitor.strikes.saturating_add(1);
            let backoff = self.backoff_for(visitor.strikes);
            visitor.blocked_until = Some(now + backoff);
            return Err(backoff);
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct RateLimitedBody {
    error: &'static str,
}

/// Axum middleware fn, layered onto the `/api/auth` route group.
pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    // Keyed by client IP when the listener provides it (see main.rs:
    // `into_make_service_with_connect_info`); a shared bucket otherwise.
    let key = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    match limiter.check(&key, Instant::now()) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            // Ceil to whole seconds — Retry-After is integral.
            let seconds = retry_after.as_secs() + u64::from(retry_after.subsec_nanos() > 0);
            (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, seconds.max(1).to_string())],
                Json(RateLimitedBody {
                    error: "too many requests",
                }),
            )
                .into_response()
        }
    }
}

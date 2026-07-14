//! Tower/axum middleware. Only cross-cutting HTTP concerns live here —
//! nothing that belongs in a use case.

mod rate_limit;

pub use rate_limit::{RateLimitConfig, RateLimiter, rate_limit};

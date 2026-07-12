//! HTTP handlers, one module per resource. Thin by rule: deserialize → call
//! use case → map to response.

pub mod health;

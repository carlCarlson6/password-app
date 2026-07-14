//! Infrastructure layer: driven adapters implementing `application` ports,
//! plus configuration loading. Nothing above this crate touches SQLx.

pub mod config;
pub mod persistence;
pub mod security;

/// Errors raised by domain invariants.
///
/// Adapters map these to protocol-level errors (HTTP 400/422, …); the domain
/// itself knows nothing about transports.
//
// Rust note: `#[derive(...)]` auto-implements traits. `Debug` enables `{:?}`
// formatting, `PartialEq`/`Eq` enable `==` (handy in tests). `thiserror::Error`
// is a derive macro that implements the standard `std::error::Error` trait,
// generating the `Display` (human-readable message) impl from the `#[error]`
// attributes below.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    // Rust note: enum variants can carry data. This one is a "struct variant"
    // with named fields. `&'static str` is a string slice that lives for the
    // whole program (a compile-time literal), so no allocation is needed.
    /// A value object rejected its input.
    #[error("invalid {field}: {reason}")]
    InvalidValue {
        field: &'static str,
        reason: &'static str,
    },

    /// An aggregate invariant would be violated by the requested operation.
    #[error("invariant violated: {0}")]
    InvariantViolation(&'static str),
}

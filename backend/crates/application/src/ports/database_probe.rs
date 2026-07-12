use async_trait::async_trait;

/// Driven port: liveness probe against the persistence backend.
///
/// Exists for the walking skeleton's health check; real repositories
/// (one per aggregate) arrive in Phases 1–2.
//
// Rust note: a `trait` is Rust's interface. Infrastructure will provide a
// concrete type implementing it; this crate never learns which one — that's
// the hexagonal boundary. `#[async_trait]` is a macro working around the fact
// that traits with `async fn` can't yet be used as trait OBJECTS (`dyn Trait`)
// natively; it rewrites each method to return a boxed Future.
#[async_trait]
pub trait DatabaseProbe: Send + Sync {
    // Rust note: `Send + Sync` above are marker traits meaning "safe to move
    // between / share across threads" — required because the web server
    // handles requests on a multi-threaded runtime.
    async fn ping(&self) -> Result<(), ProbeError>;
}

// Rust note: `String` (owned, heap-allocated) rather than `&str` here because
// the error must outlive the adapter call that produced it.
#[derive(Debug, thiserror::Error)]
#[error("database unreachable: {reason}")]
pub struct ProbeError {
    pub reason: String,
}

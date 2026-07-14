use domain::identity::UserId;

/// Driven port: mints short-lived access tokens (JWT in the adapter).
///
/// Synchronous — signing is pure CPU and fast, no I/O involved.
pub trait TokenIssuer: Send + Sync {
    fn issue(&self, user_id: &UserId, now_unix: i64) -> Result<String, TokenIssuerError>;
}

#[derive(Debug, thiserror::Error)]
#[error("token issuing failure: {0}")]
pub struct TokenIssuerError(pub String);

use async_trait::async_trait;

use domain::identity::Session;

/// Driven port: persistence for refresh-token [`Session`]s.
///
/// The rotation DECISIONS live in `Session::assess` (domain) and the
/// `RefreshSession` use case; this port only stores facts.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn insert(&self, session: &Session) -> Result<(), SessionRepositoryError>;

    /// Look up by the SHA-256 hash of a presented token — raw tokens are
    /// never stored, so a database leak cannot mint refresh cookies.
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, SessionRepositoryError>;

    /// Retire a token after rotation (it stays on disk to catch replays).
    async fn mark_used(&self, session_id: &str) -> Result<(), SessionRepositoryError>;

    /// Kill every token in a family — the reuse-detection hammer.
    async fn revoke_family(&self, family_id: &str) -> Result<(), SessionRepositoryError>;
}

#[derive(Debug, thiserror::Error)]
#[error("session store failure: {0}")]
pub struct SessionRepositoryError(pub String);

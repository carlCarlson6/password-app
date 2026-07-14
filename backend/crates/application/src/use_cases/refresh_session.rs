use std::sync::Arc;

use domain::identity::{Session, SessionAssessment};

use crate::ports::{Clock, IdGenerator, RefreshTokenVendor, SessionRepository, TokenIssuer};

pub struct RefreshedSession {
    pub access_token: String,
    /// The ROTATED refresh token — the presented one is now dead.
    pub refresh_token: String,
    pub refresh_ttl_seconds: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshSessionError {
    /// Unknown, expired, or replayed token — one indistinguishable error.
    #[error("invalid session")]
    InvalidSession,

    #[error("refresh failed: {0}")]
    Infra(String),
}

/// Use case: rotate a refresh token.
///
/// Every refresh retires the presented token and issues a new one in the
/// same family. Presenting an ALREADY-retired token is treated as theft:
/// the whole family is revoked, forcing a fresh login everywhere.
pub struct RefreshSession {
    sessions: Arc<dyn SessionRepository>,
    tokens: Arc<dyn TokenIssuer>,
    vendor: Arc<dyn RefreshTokenVendor>,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
    refresh_ttl_seconds: i64,
}

impl RefreshSession {
    pub fn new(
        sessions: Arc<dyn SessionRepository>,
        tokens: Arc<dyn TokenIssuer>,
        vendor: Arc<dyn RefreshTokenVendor>,
        ids: Arc<dyn IdGenerator>,
        clock: Arc<dyn Clock>,
        refresh_ttl_seconds: i64,
    ) -> Self {
        Self {
            sessions,
            tokens,
            vendor,
            ids,
            clock,
            refresh_ttl_seconds,
        }
    }

    pub async fn execute(
        &self,
        presented_token: &str,
    ) -> Result<RefreshedSession, RefreshSessionError> {
        let infra = |reason: String| RefreshSessionError::Infra(reason);

        let token_hash = self.vendor.hash(presented_token);
        let session = self
            .sessions
            .find_by_token_hash(&token_hash)
            .await
            .map_err(|e| infra(e.to_string()))?
            .ok_or(RefreshSessionError::InvalidSession)?;

        let now = self.clock.now_unix();
        match session.assess(now) {
            SessionAssessment::ReuseDetected => {
                // Replay of a rotated-out token: someone (client or thief)
                // holds a stale copy. Kill the entire family.
                self.sessions
                    .revoke_family(session.family_id())
                    .await
                    .map_err(|e| infra(e.to_string()))?;
                Err(RefreshSessionError::InvalidSession)
            }
            SessionAssessment::Expired => Err(RefreshSessionError::InvalidSession),
            SessionAssessment::Active => {
                self.sessions
                    .mark_used(session.id())
                    .await
                    .map_err(|e| infra(e.to_string()))?;

                let minted = self.vendor.mint();
                let next = Session::new(
                    self.ids.generate(),
                    session.family_id(),
                    session.user_id().clone(),
                    minted.token_hash,
                    false,
                    now + self.refresh_ttl_seconds,
                )
                .map_err(|e| infra(e.to_string()))?;
                self.sessions
                    .insert(&next)
                    .await
                    .map_err(|e| infra(e.to_string()))?;

                let access_token = self
                    .tokens
                    .issue(session.user_id(), now)
                    .map_err(|e| infra(e.to_string()))?;

                Ok(RefreshedSession {
                    access_token,
                    refresh_token: minted.raw,
                    refresh_ttl_seconds: self.refresh_ttl_seconds,
                })
            }
        }
    }
}

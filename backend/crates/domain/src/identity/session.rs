use crate::identity::UserId;
use crate::shared::DomainError;

/// What presenting a refresh token means, decided by pure domain logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAssessment {
    /// Token is current and unexpired: rotate it and issue a new access token.
    Active,
    /// Token expired naturally: re-authenticate.
    Expired,
    /// Token was already rotated out — someone is replaying it. The whole
    /// session family must be revoked (the rotation may have been stolen).
    ReuseDetected,
}

/// One rotating refresh token in a session family.
///
/// A LOGIN starts a family; every REFRESH retires the presented token
/// (`used`) and appends a fresh one to the same family. Only the SHA-256
/// hash of the token ever reaches this entity — the raw secret lives in the
/// client's cookie and nowhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    id: String,
    family_id: String,
    user_id: UserId,
    token_hash: String,
    used: bool,
    expires_at_unix: i64,
}

impl Session {
    pub fn new(
        id: impl Into<String>,
        family_id: impl Into<String>,
        user_id: UserId,
        token_hash: impl Into<String>,
        used: bool,
        expires_at_unix: i64,
    ) -> Result<Self, DomainError> {
        let (id, family_id, token_hash) = (id.into(), family_id.into(), token_hash.into());
        let invalid = |reason| DomainError::InvalidValue {
            field: "session",
            reason,
        };
        if id.trim().is_empty() || family_id.trim().is_empty() {
            return Err(invalid("blank id or family id"));
        }
        if token_hash.trim().is_empty() {
            return Err(invalid("blank token hash"));
        }
        Ok(Self {
            id,
            family_id,
            user_id,
            token_hash,
            used,
            expires_at_unix,
        })
    }

    /// The rotation decision — kept here (pure, instantly testable) instead
    /// of inside the use case or, worse, the SQL.
    pub fn assess(&self, now_unix: i64) -> SessionAssessment {
        if self.used {
            SessionAssessment::ReuseDetected
        } else if now_unix >= self.expires_at_unix {
            SessionAssessment::Expired
        } else {
            SessionAssessment::Active
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn token_hash(&self) -> &str {
        &self.token_hash
    }

    pub fn used(&self) -> bool {
        self.used
    }

    pub fn expires_at_unix(&self) -> i64 {
        self.expires_at_unix
    }
}

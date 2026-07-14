use std::sync::Arc;

use domain::identity::{EmailAddress, MasterPasswordHash, Session, UserAccount};

use crate::ports::{
    Clock, IdGenerator, PasswordHasher, RefreshTokenVendor, SessionRepository, TokenIssuer,
    UserRepository,
};

pub struct LoginInput {
    pub email: String,
    pub master_password_hash: Vec<u8>,
}

/// Everything a fresh browser needs to unlock locally: an access token to
/// talk to the API, a refresh token (cookie), and the wrapped key material
/// to unwrap with the Stretched Master Key it just derived.
pub struct LoggedIn {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_ttl_seconds: i64,
    pub wrapped_user_symmetric_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub wrapped_private_key: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    /// ONE error for wrong email and wrong password — responses and (as far
    /// as Argon2 allows) timings must not distinguish the two.
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("login failed: {0}")]
    Infra(String),
}

/// Use case: verify the client's login credential against the stored Argon2id
/// re-hash and start an authenticated session.
pub struct Login {
    users: Arc<dyn UserRepository>,
    sessions: Arc<dyn SessionRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenIssuer>,
    vendor: Arc<dyn RefreshTokenVendor>,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
    refresh_ttl_seconds: i64,
}

impl Login {
    #[allow(clippy::too_many_arguments)] // composition-root wiring, called once
    pub fn new(
        users: Arc<dyn UserRepository>,
        sessions: Arc<dyn SessionRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenIssuer>,
        vendor: Arc<dyn RefreshTokenVendor>,
        ids: Arc<dyn IdGenerator>,
        clock: Arc<dyn Clock>,
        refresh_ttl_seconds: i64,
    ) -> Self {
        Self {
            users,
            sessions,
            hasher,
            tokens,
            vendor,
            ids,
            clock,
            refresh_ttl_seconds,
        }
    }

    pub async fn execute(&self, input: LoginInput) -> Result<LoggedIn, LoginError> {
        let infra = |reason: String| LoginError::Infra(reason);

        // Malformed email or credential → same InvalidCredentials as any
        // other failure; validation errors must not become an oracle.
        let (Ok(email), Ok(candidate)) = (
            EmailAddress::new(input.email),
            MasterPasswordHash::new(input.master_password_hash),
        ) else {
            return Err(LoginError::InvalidCredentials);
        };

        let account = self
            .users
            .find_by_email(&email)
            .await
            .map_err(|e| infra(e.to_string()))?;

        // `stored = None` still burns a dummy Argon2 verify inside the
        // adapter — unknown email and wrong password cost the same time.
        let stored = account.as_ref().map(|a| a.credential_hash().clone());
        let verified = self
            .hasher
            .verify(&candidate, stored.as_ref())
            .await
            .map_err(|e| infra(e.to_string()))?;

        // Rust note: matching on a tuple of (Option, bool) makes the "both
        // must hold" rule exhaustive — the compiler forces the failure arm.
        let account: UserAccount = match (account, verified) {
            (Some(account), true) => account,
            _ => return Err(LoginError::InvalidCredentials),
        };

        let now = self.clock.now_unix();
        let access_token = self
            .tokens
            .issue(account.id(), now)
            .map_err(|e| infra(e.to_string()))?;

        // A login starts a NEW session family; its first token id doubles as
        // the family id. Refreshes will append to this family.
        let minted = self.vendor.mint();
        let session_id = self.ids.generate();
        let session = Session::new(
            session_id.clone(),
            session_id,
            account.id().clone(),
            minted.token_hash,
            false,
            now + self.refresh_ttl_seconds,
        )
        .map_err(|e| infra(e.to_string()))?;
        self.sessions
            .insert(&session)
            .await
            .map_err(|e| infra(e.to_string()))?;

        Ok(LoggedIn {
            access_token,
            refresh_token: minted.raw,
            refresh_ttl_seconds: self.refresh_ttl_seconds,
            wrapped_user_symmetric_key: account.wrapped_user_symmetric_key().as_bytes().to_vec(),
            public_key: account.public_key().as_bytes().to_vec(),
            wrapped_private_key: account.wrapped_private_key().as_bytes().to_vec(),
        })
    }
}

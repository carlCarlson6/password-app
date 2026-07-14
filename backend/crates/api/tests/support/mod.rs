//! Shared helpers for api tests: a full `AppState` backed by inert stubs,
//! with an injectable database probe. Auth-flow tests replace the stubs
//! with real adapters; these exist so tests of OTHER concerns (health,
//! rate limiting) can build a router without a database.

use std::sync::Arc;

use async_trait::async_trait;

use api::AppState;
use application::ports::{
    Clock, DatabaseProbe, IdGenerator, MintedRefreshToken, PasswordHasher, PasswordHasherError,
    RefreshTokenVendor, SessionRepository, SessionRepositoryError, TokenIssuer, TokenIssuerError,
    UserRepository, UserRepositoryError,
};
use application::use_cases::{CheckHealth, Login, Prelogin, RefreshSession, RegisterUser};
use domain::identity::{
    CredentialHash, EmailAddress, MasterPasswordHash, Session, UserAccount, UserId,
};

struct StubUsers;
#[async_trait]
impl UserRepository for StubUsers {
    async fn insert(&self, _: &UserAccount) -> Result<(), UserRepositoryError> {
        Ok(())
    }
    async fn find_by_email(
        &self,
        _: &EmailAddress,
    ) -> Result<Option<UserAccount>, UserRepositoryError> {
        Ok(None)
    }
}

struct StubSessions;
#[async_trait]
impl SessionRepository for StubSessions {
    async fn insert(&self, _: &Session) -> Result<(), SessionRepositoryError> {
        Ok(())
    }
    async fn find_by_token_hash(&self, _: &str) -> Result<Option<Session>, SessionRepositoryError> {
        Ok(None)
    }
    async fn mark_used(&self, _: &str) -> Result<(), SessionRepositoryError> {
        Ok(())
    }
    async fn revoke_family(&self, _: &str) -> Result<(), SessionRepositoryError> {
        Ok(())
    }
}

struct StubHasher;
#[async_trait]
impl PasswordHasher for StubHasher {
    async fn hash(&self, _: &MasterPasswordHash) -> Result<CredentialHash, PasswordHasherError> {
        Ok(CredentialHash::new("$stub$").unwrap())
    }
    async fn verify(
        &self,
        _: &MasterPasswordHash,
        _: Option<&CredentialHash>,
    ) -> Result<bool, PasswordHasherError> {
        Ok(false)
    }
}

struct StubTokens;
impl TokenIssuer for StubTokens {
    fn issue(&self, _: &UserId, _: i64) -> Result<String, TokenIssuerError> {
        Ok("stub".into())
    }
}

struct StubVendor;
impl RefreshTokenVendor for StubVendor {
    fn mint(&self) -> MintedRefreshToken {
        MintedRefreshToken {
            raw: "raw".into(),
            token_hash: "hash".into(),
        }
    }
    fn hash(&self, raw: &str) -> String {
        raw.into()
    }
}

struct StubIds;
impl IdGenerator for StubIds {
    fn generate(&self) -> String {
        "id".into()
    }
}

struct StubClock;
impl Clock for StubClock {
    fn now_unix(&self) -> i64 {
        0
    }
}

/// Full state around the given probe; all auth ports are inert stubs.
pub fn state_with_probe(probe: impl DatabaseProbe + 'static) -> AppState {
    let users = Arc::new(StubUsers);
    let sessions = Arc::new(StubSessions);
    let hasher = Arc::new(StubHasher);
    let tokens = Arc::new(StubTokens);
    let vendor = Arc::new(StubVendor);
    let ids = Arc::new(StubIds);
    let clock = Arc::new(StubClock);
    AppState {
        check_health: Arc::new(CheckHealth::new(Arc::new(probe))),
        register_user: Arc::new(RegisterUser::new(
            users.clone(),
            hasher.clone(),
            ids.clone(),
        )),
        prelogin: Arc::new(Prelogin::new(users.clone())),
        login: Arc::new(Login::new(
            users,
            sessions.clone(),
            hasher,
            tokens.clone(),
            vendor.clone(),
            ids.clone(),
            clock.clone(),
            60,
        )),
        refresh_session: Arc::new(RefreshSession::new(
            sessions, tokens, vendor, ids, clock, 60,
        )),
        cookie_secure: false,
    }
}

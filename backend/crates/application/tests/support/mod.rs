//! In-memory fakes for the driven ports, shared by the use-case tests.
//!
//! Tests exercise the crate's PUBLIC api only: every fake implements a
//! public port trait, and assertions read back through those same traits
//! (or through inspection helpers on the fakes themselves).

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

use async_trait::async_trait;

use application::ports::{
    Clock, IdGenerator, MintedRefreshToken, PasswordHasher, PasswordHasherError,
    RefreshTokenVendor, SessionRepository, SessionRepositoryError, TokenIssuer, TokenIssuerError,
    UserRepository, UserRepositoryError,
};
use domain::identity::{CredentialHash, EmailAddress, MasterPasswordHash, Session, UserAccount};

// Rust note: `Mutex` gives interior mutability behind the `&self` methods the
// port traits require; `.lock().unwrap()` is fine in tests (poisoning = bug).

#[derive(Default)]
pub struct InMemoryUsers {
    accounts: Mutex<HashMap<String, UserAccount>>,
}

impl InMemoryUsers {
    pub fn stored(&self, email: &str) -> Option<UserAccount> {
        self.accounts.lock().unwrap().get(email).cloned()
    }
}

#[async_trait]
impl UserRepository for InMemoryUsers {
    async fn insert(&self, account: &UserAccount) -> Result<(), UserRepositoryError> {
        let mut accounts = self.accounts.lock().unwrap();
        let email = account.email().as_str().to_string();
        if accounts.contains_key(&email) {
            return Err(UserRepositoryError::DuplicateEmail);
        }
        accounts.insert(email, account.clone());
        Ok(())
    }

    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<UserAccount>, UserRepositoryError> {
        Ok(self.accounts.lock().unwrap().get(email.as_str()).cloned())
    }
}

#[derive(Default)]
pub struct InMemorySessions {
    sessions: Mutex<Vec<Session>>,
}

impl InMemorySessions {
    pub fn all(&self) -> Vec<Session> {
        self.sessions.lock().unwrap().clone()
    }

    pub fn add(&self, session: Session) {
        self.sessions.lock().unwrap().push(session);
    }
}

#[async_trait]
impl SessionRepository for InMemorySessions {
    async fn insert(&self, session: &Session) -> Result<(), SessionRepositoryError> {
        self.sessions.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, SessionRepositoryError> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.token_hash() == token_hash)
            .cloned())
    }

    async fn mark_used(&self, session_id: &str) -> Result<(), SessionRepositoryError> {
        let mut sessions = self.sessions.lock().unwrap();
        let Some(found) = sessions.iter_mut().find(|s| s.id() == session_id) else {
            return Err(SessionRepositoryError("no such session".into()));
        };
        *found = Session::new(
            found.id(),
            found.family_id(),
            found.user_id().clone(),
            found.token_hash(),
            true,
            found.expires_at_unix(),
        )
        .unwrap();
        Ok(())
    }

    async fn revoke_family(&self, family_id: &str) -> Result<(), SessionRepositoryError> {
        self.sessions
            .lock()
            .unwrap()
            .retain(|s| s.family_id() != family_id);
        Ok(())
    }
}

/// Deterministic "hash": prefixes the hex of the input. Cheap, but preserves
/// the property the use cases rely on: verify(c, hash(c)) == true.
#[derive(Default)]
pub struct FakeHasher {
    pub verify_calls: AtomicUsize,
}

pub fn fake_phc(credential: &[u8]) -> String {
    let hex: String = credential.iter().map(|b| format!("{b:02x}")).collect();
    format!("$fake$${hex}")
}

#[async_trait]
impl PasswordHasher for FakeHasher {
    async fn hash(
        &self,
        credential: &MasterPasswordHash,
    ) -> Result<CredentialHash, PasswordHasherError> {
        Ok(CredentialHash::new(fake_phc(credential.as_bytes())).unwrap())
    }

    async fn verify(
        &self,
        candidate: &MasterPasswordHash,
        stored: Option<&CredentialHash>,
    ) -> Result<bool, PasswordHasherError> {
        // Counted so tests can prove the dummy path runs for unknown emails.
        self.verify_calls.fetch_add(1, Ordering::SeqCst);
        Ok(stored.is_some_and(|s| s.as_str() == fake_phc(candidate.as_bytes())))
    }
}

pub struct FakeTokens;

impl TokenIssuer for FakeTokens {
    fn issue(
        &self,
        user_id: &domain::identity::UserId,
        now_unix: i64,
    ) -> Result<String, TokenIssuerError> {
        Ok(format!("access:{}:{now_unix}", user_id.as_str()))
    }
}

/// Mints "raw-1", "raw-2", …; hashing any string prefixes it with "hashed:".
#[derive(Default)]
pub struct FakeVendor {
    counter: AtomicUsize,
}

impl RefreshTokenVendor for FakeVendor {
    fn mint(&self) -> MintedRefreshToken {
        let n = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        let raw = format!("raw-{n}");
        let token_hash = self.hash(&raw);
        MintedRefreshToken { raw, token_hash }
    }

    fn hash(&self, raw: &str) -> String {
        format!("hashed:{raw}")
    }
}

/// Generates "id-1", "id-2", …
#[derive(Default)]
pub struct SeqIds {
    counter: AtomicUsize,
}

impl IdGenerator for SeqIds {
    fn generate(&self) -> String {
        format!("id-{}", self.counter.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

/// A clock tests can move.
pub struct FixedClock {
    now: AtomicI64,
}

impl FixedClock {
    pub fn at(now: i64) -> Self {
        Self {
            now: AtomicI64::new(now),
        }
    }

    pub fn set(&self, now: i64) {
        self.now.store(now, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now_unix(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
}

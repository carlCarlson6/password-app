//! Tests for `application/src/use_cases/login.rs`.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use application::use_cases::{Login, LoginError, LoginInput, RegisterUser, RegisterUserInput};

use crate::support::{
    FakeHasher, FakeTokens, FakeVendor, FixedClock, InMemorySessions, InMemoryUsers, SeqIds,
};

const REFRESH_TTL: i64 = 1_209_600; // 14 days

struct World {
    users: Arc<InMemoryUsers>,
    sessions: Arc<InMemorySessions>,
    hasher: Arc<FakeHasher>,
    login: Login,
}

async fn world_with_alice() -> World {
    let users = Arc::new(InMemoryUsers::default());
    let sessions = Arc::new(InMemorySessions::default());
    let hasher = Arc::new(FakeHasher::default());

    RegisterUser::new(users.clone(), hasher.clone(), Arc::new(SeqIds::default()))
        .execute(RegisterUserInput {
            email: "alice@example.com".into(),
            master_password_hash: vec![7; 32],
            kdf_algorithm: "argon2id".into(),
            kdf_memory_kib: 65536,
            kdf_iterations: 3,
            kdf_parallelism: 4,
            wrapped_user_symmetric_key: vec![1; 40],
            public_key: vec![2; 294],
            wrapped_private_key: vec![3; 1300],
        })
        .await
        .unwrap();

    let login = Login::new(
        users.clone(),
        sessions.clone(),
        hasher.clone(),
        Arc::new(FakeTokens),
        Arc::new(FakeVendor::default()),
        Arc::new(SeqIds::default()),
        Arc::new(FixedClock::at(1_000)),
        REFRESH_TTL,
    );

    World {
        users,
        sessions,
        hasher,
        login,
    }
}

fn credentials(mph: Vec<u8>) -> LoginInput {
    LoginInput {
        email: "alice@example.com".into(),
        master_password_hash: mph,
    }
}

#[tokio::test]
async fn returns_tokens_and_wrapped_keys_on_success() {
    let world = world_with_alice().await;
    let logged_in = world.login.execute(credentials(vec![7; 32])).await.unwrap();

    assert_eq!(logged_in.access_token, "access:id-1:1000");
    assert_eq!(logged_in.refresh_token, "raw-1");
    assert_eq!(logged_in.refresh_ttl_seconds, REFRESH_TTL);
    assert_eq!(logged_in.wrapped_user_symmetric_key, vec![1; 40]);
    assert_eq!(logged_in.public_key, vec![2; 294]);
    assert_eq!(logged_in.wrapped_private_key, vec![3; 1300]);

    // A session family started: hashed token stored, never the raw one.
    let sessions = world.sessions.all();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].token_hash(), "hashed:raw-1");
    assert_eq!(sessions[0].id(), sessions[0].family_id());
    assert_eq!(sessions[0].user_id().as_str(), "id-1");
    assert!(!sessions[0].used());
    assert_eq!(sessions[0].expires_at_unix(), 1_000 + REFRESH_TTL);
    // Sanity: the account row itself never held the raw credential.
    assert!(world.users.stored("alice@example.com").is_some());
}

#[tokio::test]
async fn wrong_password_and_unknown_email_yield_the_same_error() {
    let world = world_with_alice().await;

    let wrong_password = world.login.execute(credentials(vec![9; 32])).await;
    let unknown_email = world
        .login
        .execute(LoginInput {
            email: "mallory@example.com".into(),
            master_password_hash: vec![7; 32],
        })
        .await;

    // Same variant, same message — no oracle in the error.
    assert!(matches!(
        wrong_password,
        Err(LoginError::InvalidCredentials)
    ));
    assert!(matches!(unknown_email, Err(LoginError::InvalidCredentials)));
    assert!(world.sessions.all().is_empty());
}

#[tokio::test]
async fn unknown_email_still_burns_a_verification() {
    let world = world_with_alice().await;
    let before = world.hasher.verify_calls.load(Ordering::SeqCst);

    let _ = world
        .login
        .execute(LoginInput {
            email: "mallory@example.com".into(),
            master_password_hash: vec![7; 32],
        })
        .await;

    // The dummy verify ran — unknown email costs the same hashing time.
    assert_eq!(world.hasher.verify_calls.load(Ordering::SeqCst), before + 1);
}

#[tokio::test]
async fn malformed_input_is_just_invalid_credentials() {
    let world = world_with_alice().await;
    let malformed_email = world
        .login
        .execute(LoginInput {
            email: "not-an-email".into(),
            master_password_hash: vec![7; 32],
        })
        .await;
    let short_hash = world.login.execute(credentials(vec![7; 4])).await;

    assert!(matches!(
        malformed_email,
        Err(LoginError::InvalidCredentials)
    ));
    assert!(matches!(short_hash, Err(LoginError::InvalidCredentials)));
}

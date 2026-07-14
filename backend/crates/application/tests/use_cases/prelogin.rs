//! Tests for `application/src/use_cases/prelogin.rs`.

use std::sync::Arc;

use application::use_cases::{Prelogin, RegisterUser, RegisterUserInput};
use domain::identity::KdfParams;

use crate::support::{FakeHasher, InMemoryUsers, SeqIds};

async fn users_with_bob() -> Arc<InMemoryUsers> {
    let users = Arc::new(InMemoryUsers::default());
    RegisterUser::new(
        users.clone(),
        Arc::new(FakeHasher::default()),
        Arc::new(SeqIds::default()),
    )
    .execute(RegisterUserInput {
        email: "bob@example.com".into(),
        master_password_hash: vec![7; 32],
        kdf_algorithm: "argon2id".into(),
        kdf_memory_kib: 32768, // NON-default, so the test can tell the difference
        kdf_iterations: 5,
        kdf_parallelism: 2,
        wrapped_user_symmetric_key: vec![1; 40],
        public_key: vec![2; 294],
        wrapped_private_key: vec![3; 1300],
    })
    .await
    .unwrap();
    users
}

#[tokio::test]
async fn returns_the_stored_params_for_a_known_email() {
    let prelogin = Prelogin::new(users_with_bob().await);
    let params = prelogin.execute("Bob@Example.COM ".into()).await.unwrap();
    assert_eq!(params.memory_kib(), 32768);
    assert_eq!(params.iterations(), 5);
    assert_eq!(params.parallelism(), 2);
}

#[tokio::test]
async fn returns_deterministic_defaults_for_an_unknown_email() {
    let prelogin = Prelogin::new(users_with_bob().await);
    let params = prelogin
        .execute("mallory@example.com".into())
        .await
        .unwrap();
    assert_eq!(params, KdfParams::default_params());
}

#[tokio::test]
async fn even_a_malformed_email_gets_the_defaults_not_an_error() {
    // Anti-enumeration: no input-shaped oracle either.
    let prelogin = Prelogin::new(users_with_bob().await);
    let params = prelogin.execute("not-an-email".into()).await.unwrap();
    assert_eq!(params, KdfParams::default_params());
}

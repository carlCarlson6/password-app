//! Tests for `infrastructure/src/persistence/sqlite_user_repository.rs`.

use application::ports::{UserRepository, UserRepositoryError};
use domain::identity::{
    CredentialHash, EmailAddress, KdfAlgorithm, KdfParams, KeyBlob, UserAccount, UserId,
};
use infrastructure::persistence::{SqliteUserRepository, connect, run_migrations};

async fn repo() -> SqliteUserRepository {
    let pool = connect("sqlite::memory:").await.expect("connect");
    run_migrations(&pool).await.expect("migrate");
    SqliteUserRepository::new(pool)
}

fn account(id: &str, email: &str) -> UserAccount {
    UserAccount::new(
        UserId::new(id).unwrap(),
        EmailAddress::new(email).unwrap(),
        CredentialHash::new("$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA").unwrap(),
        KdfParams::new(KdfAlgorithm::Argon2id, 32768, 5, 2).unwrap(),
        KeyBlob::new(vec![1; 40]).unwrap(),
        KeyBlob::new(vec![2; 294]).unwrap(),
        KeyBlob::new(vec![3; 1300]).unwrap(),
    )
}

#[tokio::test]
async fn round_trips_the_full_aggregate() {
    let repo = repo().await;
    let original = account("user-1", "alice@example.com");
    repo.insert(&original).await.unwrap();

    let found = repo
        .find_by_email(&EmailAddress::new("alice@example.com").unwrap())
        .await
        .unwrap()
        .expect("account found");

    // Value-object equality across the whole aggregate: every column
    // (including the opaque blobs) survived the round trip unchanged.
    assert_eq!(found, original);
    assert_eq!(found.kdf().memory_kib(), 32768);
}

#[tokio::test]
async fn find_returns_none_for_unknown_email() {
    let repo = repo().await;
    let found = repo
        .find_by_email(&EmailAddress::new("nobody@example.com").unwrap())
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn duplicate_email_is_a_distinct_error() {
    let repo = repo().await;
    repo.insert(&account("user-1", "alice@example.com"))
        .await
        .unwrap();

    let result = repo.insert(&account("user-2", "alice@example.com")).await;
    assert!(matches!(result, Err(UserRepositoryError::DuplicateEmail)));
}

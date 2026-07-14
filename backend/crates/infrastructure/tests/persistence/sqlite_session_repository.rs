//! Tests for `infrastructure/src/persistence/sqlite_session_repository.rs`.

use application::ports::{SessionRepository, UserRepository};
use domain::identity::{
    CredentialHash, EmailAddress, KdfParams, KeyBlob, Session, UserAccount, UserId,
};
use infrastructure::persistence::{
    SqliteSessionRepository, SqliteUserRepository, connect, run_migrations,
};

/// Sessions reference users (FK), so seed one account first.
async fn repo() -> SqliteSessionRepository {
    let pool = connect("sqlite::memory:").await.expect("connect");
    run_migrations(&pool).await.expect("migrate");
    SqliteUserRepository::new(pool.clone())
        .insert(&UserAccount::new(
            UserId::new("user-1").unwrap(),
            EmailAddress::new("alice@example.com").unwrap(),
            CredentialHash::new("$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA").unwrap(),
            KdfParams::default_params(),
            KeyBlob::new(vec![1; 40]).unwrap(),
            KeyBlob::new(vec![2; 294]).unwrap(),
            KeyBlob::new(vec![3; 1300]).unwrap(),
        ))
        .await
        .expect("seed user");
    SqliteSessionRepository::new(pool)
}

fn session(id: &str, family: &str, token_hash: &str) -> Session {
    Session::new(
        id,
        family,
        UserId::new("user-1").unwrap(),
        token_hash,
        false,
        2_000_000_000,
    )
    .unwrap()
}

#[tokio::test]
async fn round_trips_a_session_by_token_hash() {
    let repo = repo().await;
    let original = session("s1", "f1", "hash-1");
    repo.insert(&original).await.unwrap();

    let found = repo
        .find_by_token_hash("hash-1")
        .await
        .unwrap()
        .expect("session found");
    assert_eq!(found, original);

    assert!(repo.find_by_token_hash("no-such").await.unwrap().is_none());
}

#[tokio::test]
async fn mark_used_persists_and_missing_ids_fail() {
    let repo = repo().await;
    repo.insert(&session("s1", "f1", "hash-1")).await.unwrap();

    repo.mark_used("s1").await.unwrap();
    let found = repo.find_by_token_hash("hash-1").await.unwrap().unwrap();
    assert!(found.used());

    assert!(repo.mark_used("ghost").await.is_err());
}

#[tokio::test]
async fn revoke_family_deletes_all_its_sessions_and_only_those() {
    let repo = repo().await;
    repo.insert(&session("s1", "f1", "hash-1")).await.unwrap();
    repo.insert(&session("s2", "f1", "hash-2")).await.unwrap();
    repo.insert(&session("s3", "f2", "hash-3")).await.unwrap();

    repo.revoke_family("f1").await.unwrap();

    assert!(repo.find_by_token_hash("hash-1").await.unwrap().is_none());
    assert!(repo.find_by_token_hash("hash-2").await.unwrap().is_none());
    // The other family survives.
    assert!(repo.find_by_token_hash("hash-3").await.unwrap().is_some());
}

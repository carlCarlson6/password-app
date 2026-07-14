//! Tests for `infrastructure/src/security/argon2_password_hasher.rs`.

use application::ports::PasswordHasher;
use domain::identity::MasterPasswordHash;
use infrastructure::security::Argon2PasswordHasher;

fn credential(fill: u8) -> MasterPasswordHash {
    MasterPasswordHash::new(vec![fill; 32]).unwrap()
}

#[tokio::test]
async fn hash_then_verify_round_trips() {
    let hasher = Argon2PasswordHasher::new().unwrap();
    let stored = hasher.hash(&credential(7)).await.unwrap();

    // A PHC-format Argon2id string, never the raw credential bytes.
    assert!(stored.as_str().starts_with("$argon2id$"));

    assert!(hasher.verify(&credential(7), Some(&stored)).await.unwrap());
    assert!(!hasher.verify(&credential(9), Some(&stored)).await.unwrap());
}

#[tokio::test]
async fn equal_credentials_hash_differently_thanks_to_random_salts() {
    let hasher = Argon2PasswordHasher::new().unwrap();
    let first = hasher.hash(&credential(7)).await.unwrap();
    let second = hasher.hash(&credential(7)).await.unwrap();
    assert_ne!(first.as_str(), second.as_str());
}

#[tokio::test]
async fn the_dummy_path_burns_work_but_never_authenticates() {
    let hasher = Argon2PasswordHasher::new().unwrap();
    // `None` = "no such user": must come back false without erroring —
    // and (by construction) after a real Argon2 verification ran.
    assert!(!hasher.verify(&credential(7), None).await.unwrap());
}

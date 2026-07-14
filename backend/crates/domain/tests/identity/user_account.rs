//! Tests for `domain/src/identity/user_account.rs`.

use domain::identity::{CredentialHash, EmailAddress, KdfParams, KeyBlob, UserAccount, UserId};

#[test]
fn assembles_from_validated_value_objects_and_exposes_them() {
    let account = UserAccount::new(
        UserId::new("user-1").unwrap(),
        EmailAddress::new("alice@example.com").unwrap(),
        CredentialHash::new("$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA").unwrap(),
        KdfParams::default_params(),
        KeyBlob::new(vec![1; 40]).unwrap(),
        KeyBlob::new(vec![2; 294]).unwrap(),
        KeyBlob::new(vec![3; 1300]).unwrap(),
    );

    assert_eq!(account.id().as_str(), "user-1");
    assert_eq!(account.email().as_str(), "alice@example.com");
    assert!(account.credential_hash().as_str().starts_with("$argon2id$"));
    assert_eq!(account.kdf(), KdfParams::default_params());
    assert_eq!(
        account.wrapped_user_symmetric_key().as_bytes(),
        &[1; 40][..]
    );
    assert_eq!(account.public_key().as_bytes(), &[2; 294][..]);
    assert_eq!(account.wrapped_private_key().as_bytes(), &[3; 1300][..]);
}

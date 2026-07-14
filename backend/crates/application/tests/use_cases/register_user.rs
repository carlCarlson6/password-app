//! Tests for `application/src/use_cases/register_user.rs`.

use std::sync::Arc;

use application::use_cases::{RegisterUser, RegisterUserError, RegisterUserInput};

use crate::support::{FakeHasher, InMemoryUsers, SeqIds, fake_phc};

fn valid_input() -> RegisterUserInput {
    RegisterUserInput {
        email: "Alice@Example.com".into(),
        master_password_hash: vec![7; 32],
        kdf_algorithm: "argon2id".into(),
        kdf_memory_kib: 65536,
        kdf_iterations: 3,
        kdf_parallelism: 4,
        wrapped_user_symmetric_key: vec![1; 40],
        public_key: vec![2; 294],
        wrapped_private_key: vec![3; 1300],
    }
}

fn use_case(users: &Arc<InMemoryUsers>) -> RegisterUser {
    RegisterUser::new(
        users.clone(),
        Arc::new(FakeHasher::default()),
        Arc::new(SeqIds::default()),
    )
}

#[tokio::test]
async fn stores_the_account_with_a_rehashed_credential() {
    let users = Arc::new(InMemoryUsers::default());
    use_case(&users).execute(valid_input()).await.unwrap();

    let stored = users.stored("alice@example.com").expect("account stored");
    assert_eq!(stored.id().as_str(), "id-1");
    assert_eq!(stored.email().as_str(), "alice@example.com"); // normalized
    // The RE-HASH is stored — never the raw client credential.
    assert_eq!(stored.credential_hash().as_str(), fake_phc(&[7; 32]));
    assert_eq!(stored.kdf().memory_kib(), 65536);
    assert_eq!(stored.wrapped_user_symmetric_key().as_bytes(), &[1; 40][..]);
    assert_eq!(stored.public_key().as_bytes(), &[2; 294][..]);
    assert_eq!(stored.wrapped_private_key().as_bytes(), &[3; 1300][..]);
}

#[tokio::test]
async fn duplicate_email_reports_success_and_keeps_the_original() {
    let users = Arc::new(InMemoryUsers::default());
    let register = use_case(&users);

    register.execute(valid_input()).await.unwrap();

    let mut second = valid_input();
    second.master_password_hash = vec![9; 32];
    // Anti-enumeration: the duplicate is indistinguishable from success…
    register.execute(second).await.unwrap();

    // …and the original account is untouched.
    let stored = users.stored("alice@example.com").unwrap();
    assert_eq!(stored.credential_hash().as_str(), fake_phc(&[7; 32]));
}

#[tokio::test]
async fn rejects_invalid_input() {
    let users = Arc::new(InMemoryUsers::default());
    let register = use_case(&users);

    let broken: Vec<Box<dyn Fn(&mut RegisterUserInput)>> = vec![
        Box::new(|i| i.email = "not-an-email".into()),
        Box::new(|i| i.master_password_hash = vec![1; 4]), // too short
        Box::new(|i| i.kdf_algorithm = "md5".into()),
        Box::new(|i| i.kdf_memory_kib = 16), // dangerously weak
        Box::new(|i| i.kdf_iterations = 0),
        Box::new(|i| i.wrapped_user_symmetric_key = vec![]),
        Box::new(|i| i.public_key = vec![]),
        Box::new(|i| i.wrapped_private_key = vec![]),
    ];

    for (n, mutate) in broken.iter().enumerate() {
        let mut input = valid_input();
        mutate(&mut input);
        let result = register.execute(input).await;
        assert!(
            matches!(result, Err(RegisterUserError::Invalid(_))),
            "case {n} should be Invalid, got {result:?}"
        );
    }
    assert!(users.stored("alice@example.com").is_none());
}

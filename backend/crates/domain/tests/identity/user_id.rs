//! Tests for `domain/src/identity/user_id.rs`.

use domain::identity::UserId;

#[test]
fn accepts_any_non_blank_id() {
    let id = UserId::new("0193b6c8-user").unwrap();
    assert_eq!(id.as_str(), "0193b6c8-user");
    assert_eq!(id.to_string(), "0193b6c8-user");
}

#[test]
fn rejects_blank_ids() {
    assert!(UserId::new("").is_err());
    assert!(UserId::new("   ").is_err());
}

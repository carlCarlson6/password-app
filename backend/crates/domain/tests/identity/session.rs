//! Tests for `domain/src/identity/session.rs`.

use domain::identity::{Session, SessionAssessment, UserId};

fn session(used: bool, expires_at: i64) -> Session {
    Session::new(
        "session-1",
        "family-1",
        UserId::new("user-1").unwrap(),
        "token-hash",
        used,
        expires_at,
    )
    .unwrap()
}

#[test]
fn an_unused_unexpired_token_is_active() {
    assert_eq!(session(false, 1000).assess(999), SessionAssessment::Active);
}

#[test]
fn an_unused_token_past_expiry_is_expired() {
    assert_eq!(
        session(false, 1000).assess(1000),
        SessionAssessment::Expired
    );
    assert_eq!(
        session(false, 1000).assess(5000),
        SessionAssessment::Expired
    );
}

#[test]
fn a_rotated_out_token_is_reuse_even_if_unexpired() {
    // Reuse detection outranks expiry: a replayed token must nuke the family.
    assert_eq!(
        session(true, 1000).assess(1),
        SessionAssessment::ReuseDetected
    );
    assert_eq!(
        session(true, 1000).assess(5000),
        SessionAssessment::ReuseDetected
    );
}

#[test]
fn exposes_its_fields() {
    let s = session(false, 42);
    assert_eq!(s.id(), "session-1");
    assert_eq!(s.family_id(), "family-1");
    assert_eq!(s.user_id().as_str(), "user-1");
    assert_eq!(s.token_hash(), "token-hash");
    assert!(!s.used());
    assert_eq!(s.expires_at_unix(), 42);
}

#[test]
fn rejects_blank_identifiers() {
    let user = UserId::new("user-1").unwrap();
    assert!(Session::new("", "family", user.clone(), "hash", false, 1).is_err());
    assert!(Session::new("id", " ", user.clone(), "hash", false, 1).is_err());
    assert!(Session::new("id", "family", user, "", false, 1).is_err());
}

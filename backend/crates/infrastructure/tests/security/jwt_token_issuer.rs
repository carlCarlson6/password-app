//! Tests for `infrastructure/src/security/jwt_token_issuer.rs`.

use std::time::{SystemTime, UNIX_EPOCH};

use application::ports::TokenIssuer;
use domain::identity::UserId;
use infrastructure::security::JwtTokenIssuer;

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[test]
fn issues_a_verifiable_token_with_subject_and_expiry() {
    let issuer = JwtTokenIssuer::new(b"test-secret", 900);
    let now = now_unix();

    let token = issuer.issue(&UserId::new("user-1").unwrap(), now).unwrap();
    let claims = issuer.verify(&token).unwrap();

    assert_eq!(claims.sub, "user-1");
    assert_eq!(claims.iat, now);
    assert_eq!(claims.exp, now + 900);
}

#[test]
fn rejects_tokens_signed_with_a_different_secret() {
    let issuer = JwtTokenIssuer::new(b"test-secret", 900);
    let forger = JwtTokenIssuer::new(b"other-secret", 900);

    let forged = forger
        .issue(&UserId::new("user-1").unwrap(), now_unix())
        .unwrap();
    assert!(issuer.verify(&forged).is_err());
    assert!(issuer.verify("not-even-a-jwt").is_err());
}

#[test]
fn rejects_expired_tokens() {
    let issuer = JwtTokenIssuer::new(b"test-secret", 900);
    // Issued far enough in the past that exp is beyond any leeway.
    let stale = issuer
        .issue(&UserId::new("user-1").unwrap(), now_unix() - 10_000)
        .unwrap();
    assert!(issuer.verify(&stale).is_err());
}

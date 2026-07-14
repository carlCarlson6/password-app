//! Tests for `domain/src/identity/credential_hash.rs`.

use domain::identity::CredentialHash;

#[test]
fn accepts_phc_format_strings() {
    let phc = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA";
    assert_eq!(CredentialHash::new(phc).unwrap().as_str(), phc);
}

#[test]
fn rejects_non_phc_input() {
    assert!(CredentialHash::new("").is_err());
    assert!(CredentialHash::new("plaintext-or-hex").is_err());
}

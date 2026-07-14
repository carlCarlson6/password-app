//! Tests for `infrastructure/src/security/sha256_refresh_token_vendor.rs`.

use application::ports::RefreshTokenVendor;
use infrastructure::security::Sha256RefreshTokenVendor;

#[test]
fn minted_tokens_look_up_by_their_own_hash() {
    let vendor = Sha256RefreshTokenVendor;
    let minted = vendor.mint();
    // The identity the session lookup depends on.
    assert_eq!(vendor.hash(&minted.raw), minted.token_hash);
    // 32 random bytes → 43 chars of unpadded base64url (cookie-safe).
    assert_eq!(minted.raw.len(), 43);
    assert!(!minted.raw.contains(['+', '/', '='])); // url-safe alphabet
    // SHA-256 hex digest.
    assert_eq!(minted.token_hash.len(), 64);
}

#[test]
fn mints_are_unique_and_hashing_is_deterministic() {
    let vendor = Sha256RefreshTokenVendor;
    assert_ne!(vendor.mint().raw, vendor.mint().raw);
    assert_eq!(vendor.hash("abc"), vendor.hash("abc"));
    assert_ne!(vendor.hash("abc"), vendor.hash("abd"));
}

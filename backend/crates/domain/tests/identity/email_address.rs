//! Tests for `domain/src/identity/email_address.rs`.

use domain::identity::EmailAddress;

#[test]
fn normalizes_case_and_whitespace() {
    // Rust note: `.unwrap()` extracts the Ok value and panics on Err —
    // fine in tests, avoided in production code.
    let email = EmailAddress::new("  Alice@Example.COM ").unwrap();
    assert_eq!(email.as_str(), "alice@example.com");
}

#[test]
fn equality_is_by_value() {
    assert_eq!(
        EmailAddress::new("a@example.com").unwrap(),
        EmailAddress::new("A@EXAMPLE.COM").unwrap()
    );
}

#[test]
fn rejects_malformed_addresses() {
    for raw in [
        "",
        "no-at-sign",
        "@example.com",
        "a@nodot",
        "a@.com",
        "a@com.",
    ] {
        assert!(EmailAddress::new(raw).is_err(), "should reject {raw:?}");
    }
}

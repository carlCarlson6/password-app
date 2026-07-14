//! Tests for `infrastructure/src/security/uuid_id_generator.rs`.

use application::ports::IdGenerator;
use infrastructure::security::UuidGenerator;

#[test]
fn generates_unique_uuid_shaped_ids() {
    let ids = UuidGenerator;
    let first = ids.generate();
    let second = ids.generate();

    assert_ne!(first, second);
    // Canonical UUID text form: 36 chars, hyphens at fixed positions.
    assert_eq!(first.len(), 36);
    assert_eq!(first.chars().filter(|c| *c == '-').count(), 4);
}

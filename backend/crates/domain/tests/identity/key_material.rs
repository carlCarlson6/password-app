//! Tests for `domain/src/identity/key_material.rs`.

use domain::identity::{KeyBlob, MasterPasswordHash};

#[test]
fn key_blob_holds_opaque_bytes() {
    let blob = KeyBlob::new(vec![1, 2, 3]).unwrap();
    assert_eq!(blob.as_bytes(), &[1, 2, 3]);
}

#[test]
fn key_blob_rejects_empty_and_oversized_input() {
    assert!(KeyBlob::new(vec![]).is_err());
    assert!(KeyBlob::new(vec![0; KeyBlob::MAX_LEN]).is_ok());
    assert!(KeyBlob::new(vec![0; KeyBlob::MAX_LEN + 1]).is_err());
}

#[test]
fn master_password_hash_accepts_typical_digest_lengths() {
    // 32 bytes = a 256-bit Argon2id output, the client's actual shape.
    let hash = MasterPasswordHash::new(vec![7; 32]).unwrap();
    assert_eq!(hash.as_bytes().len(), 32);
}

#[test]
fn master_password_hash_rejects_out_of_range_lengths() {
    assert!(MasterPasswordHash::new(vec![7; 15]).is_err());
    assert!(MasterPasswordHash::new(vec![7; 16]).is_ok());
    assert!(MasterPasswordHash::new(vec![7; 128]).is_ok());
    assert!(MasterPasswordHash::new(vec![7; 129]).is_err());
}

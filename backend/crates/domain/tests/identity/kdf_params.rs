//! Tests for `domain/src/identity/kdf_params.rs`.

use domain::identity::{KdfAlgorithm, KdfParams};

#[test]
fn default_params_match_the_documented_signup_defaults() {
    let params = KdfParams::default_params();
    assert_eq!(params.algorithm(), KdfAlgorithm::Argon2id);
    assert_eq!(params.memory_kib(), 65536);
    assert_eq!(params.iterations(), 3);
    assert_eq!(params.parallelism(), 4);
}

#[test]
fn accepts_values_within_bounds() {
    let params = KdfParams::new(KdfAlgorithm::Argon2id, 32768, 4, 2).unwrap();
    assert_eq!(params.memory_kib(), 32768);
    assert_eq!(params.iterations(), 4);
    assert_eq!(params.parallelism(), 2);
}

#[test]
fn rejects_out_of_bounds_values() {
    let cases = [
        (1024, 3, 4),            // memory too small (DoS-cheap for attackers)
        (2 * 1024 * 1024, 3, 4), // memory too large (DoS on the client)
        (65536, 0, 4),           // zero iterations
        (65536, 17, 4),          // too many iterations
        (65536, 3, 0),           // zero lanes
        (65536, 3, 17),          // too many lanes
    ];
    for (memory, iterations, parallelism) in cases {
        assert!(
            KdfParams::new(KdfAlgorithm::Argon2id, memory, iterations, parallelism).is_err(),
            "should reject m={memory} t={iterations} p={parallelism}"
        );
    }
}

#[test]
fn algorithm_round_trips_through_its_wire_name() {
    assert_eq!(KdfAlgorithm::Argon2id.as_str(), "argon2id");
    assert_eq!(
        KdfAlgorithm::parse("argon2id").unwrap(),
        KdfAlgorithm::Argon2id
    );
    assert!(KdfAlgorithm::parse("scrypt").is_err());
    assert!(KdfAlgorithm::parse("ARGON2ID").is_err());
}

use crate::shared::DomainError;

/// Supported client-side key-derivation algorithms.
///
/// Only Argon2id today; an enum (not a string) so adding, say, OPAQUE later
/// is a compiler-checked change everywhere these params are consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfAlgorithm {
    Argon2id,
}

impl KdfAlgorithm {
    /// Canonical wire name (what prelogin/register exchange).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Argon2id => "argon2id",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "argon2id" => Ok(Self::Argon2id),
            _ => Err(DomainError::InvalidValue {
                field: "kdf algorithm",
                reason: "unsupported (expected \"argon2id\")",
            }),
        }
    }
}

/// Parameters the CLIENT uses to derive its Master Key (see README key
/// hierarchy). Stored per user and served by prelogin; also served — as
/// deterministic defaults — for unknown emails, to block enumeration.
///
/// Bounds reject configurations that are either dangerously weak or
/// denial-of-service sized. They mirror sane Argon2id ranges, with the
/// signup default at m=64 MiB, t=3, p=4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    algorithm: KdfAlgorithm,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
}

impl KdfParams {
    pub const MIN_MEMORY_KIB: u32 = 8 * 1024; // 8 MiB
    pub const MAX_MEMORY_KIB: u32 = 1024 * 1024; // 1 GiB
    pub const MAX_ITERATIONS: u32 = 16;
    pub const MAX_PARALLELISM: u32 = 16;

    pub fn new(
        algorithm: KdfAlgorithm,
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
    ) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::InvalidValue {
            field: "kdf params",
            reason,
        };
        // Rust note: `!(a..=b).contains(&x)` — ranges are first-class values;
        // `contains` borrows because ranges are generic over any ordered type.
        if !(Self::MIN_MEMORY_KIB..=Self::MAX_MEMORY_KIB).contains(&memory_kib) {
            return Err(invalid("memory outside 8 MiB..=1 GiB"));
        }
        if !(1..=Self::MAX_ITERATIONS).contains(&iterations) {
            return Err(invalid("iterations outside 1..=16"));
        }
        if !(1..=Self::MAX_PARALLELISM).contains(&parallelism) {
            return Err(invalid("parallelism outside 1..=16"));
        }
        Ok(Self {
            algorithm,
            memory_kib,
            iterations,
            parallelism,
        })
    }

    /// The deterministic signup default, ALSO returned by prelogin for
    /// unknown emails so their response is indistinguishable from a real one.
    pub fn default_params() -> Self {
        Self {
            algorithm: KdfAlgorithm::Argon2id,
            memory_kib: 64 * 1024,
            iterations: 3,
            parallelism: 4,
        }
    }

    pub fn algorithm(&self) -> KdfAlgorithm {
        self.algorithm
    }

    pub fn memory_kib(&self) -> u32 {
        self.memory_kib
    }

    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    pub fn parallelism(&self) -> u32 {
        self.parallelism
    }
}

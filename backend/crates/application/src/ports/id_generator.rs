/// Driven port: mints unique identifiers (UUIDv4 in the adapter).
///
/// Randomness is I/O-by-spirit, so neither domain nor use cases call an RNG
/// directly — tests inject predictable sequences instead.
pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> String;
}

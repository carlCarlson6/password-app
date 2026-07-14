/// Driven port: the current time as Unix seconds.
///
/// A port (not `SystemTime::now()` inline) so use-case tests can freeze or
/// travel time deterministically — expiry logic is untestable otherwise.
pub trait Clock: Send + Sync {
    fn now_unix(&self) -> i64;
}

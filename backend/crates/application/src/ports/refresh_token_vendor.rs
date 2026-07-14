/// A freshly minted refresh token: the raw secret goes into the client's
/// httpOnly cookie; only the hash is ever persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintedRefreshToken {
    pub raw: String,
    pub token_hash: String,
}

/// Driven port: mints cryptographically random refresh tokens and computes
/// the storage/lookup hash of a presented one (SHA-256 in the adapter).
pub trait RefreshTokenVendor: Send + Sync {
    fn mint(&self) -> MintedRefreshToken;

    /// Must satisfy `hash(&mint().raw) == mint().token_hash` — that identity
    /// is what lets a presented cookie find its stored session.
    fn hash(&self, raw: &str) -> String;
}

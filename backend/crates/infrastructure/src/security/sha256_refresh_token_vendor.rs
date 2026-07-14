use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

use application::ports::{MintedRefreshToken, RefreshTokenVendor};

/// Adapter for the [`RefreshTokenVendor`] port: 256-bit random tokens
/// (base64url, cookie-safe) stored as SHA-256 hex digests.
///
/// SHA-256 (not Argon2) is right here: the input is a 256-bit random secret,
/// not a human password — brute force is hopeless, so the hash only needs to
/// be one-way, not slow.
pub struct Sha256RefreshTokenVendor;

impl RefreshTokenVendor for Sha256RefreshTokenVendor {
    fn mint(&self) -> MintedRefreshToken {
        let mut secret = [0u8; 32];
        // Rust note: `rand::rngs::OsRng` pulls from the operating system's
        // CSPRNG — the right source for security tokens; it cannot fail
        // without the OS itself being broken.
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let raw = URL_SAFE_NO_PAD.encode(secret);
        let token_hash = self.hash(&raw);
        MintedRefreshToken { raw, token_hash }
    }

    fn hash(&self, raw: &str) -> String {
        let digest = Sha256::digest(raw.as_bytes());
        // Rust note: fold the bytes into lowercase hex; `format!("{b:02x}")`
        // zero-pads each byte to two hex digits.
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }
}

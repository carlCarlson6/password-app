use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use application::ports::{TokenIssuer, TokenIssuerError};
use domain::identity::UserId;

/// Standard JWT claims for an access token: subject + issued-at + expiry.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

/// HS256 JWT adapter for the [`TokenIssuer`] port. Short-lived by design;
/// long-lived authentication rides on the rotating refresh cookie instead.
pub struct JwtTokenIssuer {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl_seconds: i64,
}

impl JwtTokenIssuer {
    pub fn new(secret: &[u8], ttl_seconds: i64) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            ttl_seconds,
        }
    }

    /// Decode + validate a token (signature and expiry). Used by tests today
    /// and by the auth extractor when protected routes arrive in Phase 2.
    pub fn verify(&self, token: &str) -> Result<AccessTokenClaims, TokenIssuerError> {
        decode::<AccessTokenClaims>(token, &self.decoding, &Validation::new(Algorithm::HS256))
            .map(|data| data.claims)
            .map_err(|error| TokenIssuerError(error.to_string()))
    }
}

impl TokenIssuer for JwtTokenIssuer {
    fn issue(&self, user_id: &UserId, now_unix: i64) -> Result<String, TokenIssuerError> {
        let claims = AccessTokenClaims {
            sub: user_id.as_str().to_string(),
            iat: now_unix,
            exp: now_unix + self.ttl_seconds,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(|error| TokenIssuerError(error.to_string()))
    }
}

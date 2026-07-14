/// Runtime configuration, loaded from the environment with dev-friendly
/// defaults. Secrets management hardening lands in Phase 4.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_addr: String,
    /// HS256 signing secret for access JWTs. The default is for LOCAL DEV
    /// ONLY — deployment must set `JWT_SECRET`.
    pub jwt_secret: String,
    /// Whether the refresh cookie carries the `Secure` attribute. Defaults
    /// off so plain-http local dev works; production must set it.
    pub cookie_secure: bool,
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env_or("DATABASE_URL", "sqlite://data/app.db"),
            bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
            jwt_secret: env_or("JWT_SECRET", "dev-only-jwt-secret-do-not-deploy"),
            cookie_secure: env_or("COOKIE_SECURE", "false") == "true",
            access_token_ttl_seconds: env_i64("ACCESS_TOKEN_TTL_SECONDS", 15 * 60),
            refresh_token_ttl_seconds: env_i64("REFRESH_TOKEN_TTL_SECONDS", 14 * 24 * 60 * 60),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_i64(key: &str, default: i64) -> i64 {
    // Rust note: `ok()`/`and_then` chain Options — a missing OR unparsable
    // variable both fall back to the default.
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

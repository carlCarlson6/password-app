/// Runtime configuration, loaded from the environment with dev-friendly
/// defaults. Secrets management hardening lands in Phase 4.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_addr: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env_or("DATABASE_URL", "sqlite://data/app.db"),
            bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

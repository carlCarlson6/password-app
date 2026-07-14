//! Composition root: load config, build adapters, wire use cases, serve.

use std::net::SocketAddr;
use std::sync::Arc;

use application::use_cases::{CheckHealth, Login, Prelogin, RefreshSession, RegisterUser};
use infrastructure::config::AppConfig;
use infrastructure::persistence::{
    SqliteDatabaseProbe, SqliteSessionRepository, SqliteUserRepository, connect, run_migrations,
};
use infrastructure::security::{
    Argon2PasswordHasher, JwtTokenIssuer, Sha256RefreshTokenVendor, SystemClock, UuidGenerator,
};

use api::{AppState, build_router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Rust note: anyhow::Result lets `?` bubble up ANY error type from main;
    // fine at the outermost edge, never inside domain/application code.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let config = AppConfig::from_env();

    let pool = connect(&config.database_url).await?;
    run_migrations(&pool).await?;

    // Hexagonal wiring, innermost out: adapters → use cases → HTTP state.
    // Rust note: one `Arc` per adapter, `.clone()`d into each use case that
    // needs it — clones are pointer copies, the adapter exists once.
    let users = Arc::new(SqliteUserRepository::new(pool.clone()));
    let sessions = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let hasher = Arc::new(Argon2PasswordHasher::new()?); // one Argon2 at boot (dummy hash)
    let tokens = Arc::new(JwtTokenIssuer::new(
        config.jwt_secret.as_bytes(),
        config.access_token_ttl_seconds,
    ));
    let vendor = Arc::new(Sha256RefreshTokenVendor);
    let ids = Arc::new(UuidGenerator);
    let clock = Arc::new(SystemClock);

    let state = AppState {
        check_health: Arc::new(CheckHealth::new(Arc::new(SqliteDatabaseProbe::new(pool)))),
        register_user: Arc::new(RegisterUser::new(
            users.clone(),
            hasher.clone(),
            ids.clone(),
        )),
        prelogin: Arc::new(Prelogin::new(users.clone())),
        login: Arc::new(Login::new(
            users,
            sessions.clone(),
            hasher,
            tokens.clone(),
            vendor.clone(),
            ids.clone(),
            clock.clone(),
            config.refresh_token_ttl_seconds,
        )),
        refresh_session: Arc::new(RefreshSession::new(
            sessions,
            tokens,
            vendor,
            ids,
            clock,
            config.refresh_token_ttl_seconds,
        )),
        cookie_secure: config.cookie_secure,
    };

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "api listening");
    // `with_connect_info` exposes each client's SocketAddr to the rate
    // limiter, which keys its buckets by IP.
    axum::serve(
        listener,
        build_router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    // Rust note: `.ok()` discards a Result we can't act on — if installing the
    // Ctrl-C handler fails we simply won't shut down gracefully.
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("shutdown signal received");
}

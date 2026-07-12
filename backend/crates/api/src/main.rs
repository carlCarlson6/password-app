//! Composition root: load config, build adapters, wire use cases, serve.

use std::sync::Arc;

use application::use_cases::CheckHealth;
use infrastructure::config::AppConfig;
use infrastructure::persistence::{SqliteDatabaseProbe, connect, run_migrations};

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

    // Hexagonal wiring, innermost out: adapter → use case → HTTP state.
    let check_health = CheckHealth::new(Arc::new(SqliteDatabaseProbe::new(pool)));
    let state = AppState {
        check_health: Arc::new(check_health),
    };

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "api listening");
    axum::serve(listener, build_router(state))
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

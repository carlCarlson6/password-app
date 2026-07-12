//! Tests for `infrastructure/src/persistence/sqlite_probe.rs`.

use application::ports::DatabaseProbe;
use infrastructure::persistence::{SqliteDatabaseProbe, connect};

#[tokio::test]
async fn ping_succeeds_against_a_live_database() {
    let pool = connect("sqlite::memory:").await.expect("connect");
    let probe = SqliteDatabaseProbe::new(pool);
    assert!(probe.ping().await.is_ok());
}

#[tokio::test]
async fn ping_fails_once_the_pool_is_closed() {
    let pool = connect("sqlite::memory:").await.expect("connect");
    let probe = SqliteDatabaseProbe::new(pool.clone());
    pool.close().await;
    assert!(probe.ping().await.is_err());
}

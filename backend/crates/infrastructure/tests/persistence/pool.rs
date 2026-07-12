//! Tests for `infrastructure/src/persistence/pool.rs`.

use infrastructure::persistence::{connect, run_migrations};

#[tokio::test]
async fn connects_and_migrates_an_in_memory_database() {
    // Rust note: `sqlite::memory:` gives each test an isolated throwaway DB —
    // no files touched, no cleanup needed.
    let pool = connect("sqlite::memory:").await.expect("connect");
    run_migrations(&pool)
        .await
        .expect("migrations apply cleanly");

    // The migrator records applied versions in its own table; its presence
    // proves the migration machinery ran end to end.
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("migrations table exists");
    assert!(count >= 1);
}

#[tokio::test]
async fn fails_when_the_database_directory_does_not_exist() {
    // `create_if_missing` creates the FILE, never parent directories — the
    // deployment (or dev setup) owns the data directory.
    assert!(
        connect("sqlite:///nonexistent-dir-for-test/app.db")
            .await
            .is_err()
    );
}

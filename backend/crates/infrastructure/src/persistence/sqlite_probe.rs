use async_trait::async_trait;
use sqlx::SqlitePool;

use application::ports::{DatabaseProbe, ProbeError};

/// SQLite adapter for the [`DatabaseProbe`] driven port.
//
// Rust note: this is the hexagonal "adapter" half — a concrete type
// implementing the port trait declared in `application`. The pool is cheap to
// clone (it's an `Arc` internally), so adapters hold their own handle.
pub struct SqliteDatabaseProbe {
    pool: SqlitePool,
}

impl SqliteDatabaseProbe {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DatabaseProbe for SqliteDatabaseProbe {
    async fn ping(&self) -> Result<(), ProbeError> {
        // Rust note: `map`/`map_err` transform the Ok/Err halves of a Result
        // without an if/match — here: discard the row count, stringify the error.
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| ProbeError {
                reason: error.to_string(),
            })
    }
}

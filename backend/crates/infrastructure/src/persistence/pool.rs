use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;

// Rust note: `static` is a value with a fixed memory location for the whole
// program. `sqlx::migrate!` embeds every file under `migrations/` into the
// binary at COMPILE time, so deploys can never forget the .sql files.
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Open (creating if missing) the SQLite database behind `database_url`.
//
// Rust note: `async fn` returns a Future — nothing runs until it is
// `.await`ed by the caller (the api crate's Tokio runtime). This crate has
// async code, unlike `domain`, because talking to a database IS I/O.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Rust note: `?` propagates the error to the caller early — like a
    // `throw`, except the possibility is visible in the return type.
    // `parse` comes from the `FromStr` trait; the target type is inferred.
    let options: SqliteConnectOptions = database_url.parse()?;
    SqlitePool::connect_with(options.create_if_missing(true)).await
}

/// Apply pending migrations. Called once at startup by the composition root.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Rust note: `Ok(...?)` converts the error type: `MigrateError` becomes
    // `sqlx::Error` through its `From` impl, so callers handle one error type.
    Ok(MIGRATOR.run(pool).await?)
}

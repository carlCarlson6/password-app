//! Persistence adapters: SQLx + SQLite — the connection pool, embedded
//! migrations, and one repository per aggregate.
//!
//! [`DatabaseProbe`]: application::ports::DatabaseProbe

mod pool;
mod sqlite_probe;
mod sqlite_session_repository;
mod sqlite_user_repository;

pub use pool::{connect, run_migrations};
pub use sqlite_probe::SqliteDatabaseProbe;
pub use sqlite_session_repository::SqliteSessionRepository;
pub use sqlite_user_repository::SqliteUserRepository;

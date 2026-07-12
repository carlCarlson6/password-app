//! Persistence adapters: SQLx + SQLite. Repositories (one per aggregate)
//! arrive in Phases 1–2; for the walking skeleton this wires a connection
//! pool, migrations, and the [`DatabaseProbe`] adapter.
//!
//! [`DatabaseProbe`]: application::ports::DatabaseProbe

mod pool;
mod sqlite_probe;

pub use pool::{connect, run_migrations};
pub use sqlite_probe::SqliteDatabaseProbe;

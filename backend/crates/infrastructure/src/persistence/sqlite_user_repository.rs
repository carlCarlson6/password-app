use async_trait::async_trait;
use sqlx::SqlitePool;

use application::ports::{UserRepository, UserRepositoryError};
use domain::identity::{
    CredentialHash, EmailAddress, KdfAlgorithm, KdfParams, KeyBlob, UserAccount, UserId,
};

/// SQLite adapter for the [`UserRepository`] port — the ONLY doorway to the
/// `users` table (one repository per aggregate).
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Rust note: a "row" struct + `sqlx::FromRow` derive maps columns to fields
// by name. It stays private: the wire between SQL and the domain, nothing more.
#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    credential_hash: String,
    kdf_algorithm: String,
    kdf_memory_kib: i64,
    kdf_iterations: i64,
    kdf_parallelism: i64,
    wrapped_user_symmetric_key: Vec<u8>,
    public_key: Vec<u8>,
    wrapped_private_key: Vec<u8>,
}

impl UserRow {
    /// Rehydrate the aggregate, re-running every value-object invariant —
    /// a corrupt row surfaces as a Store error, never as a bad domain value.
    fn into_account(self) -> Result<UserAccount, UserRepositoryError> {
        let corrupt = |what: &str| UserRepositoryError::Store(format!("corrupt user row: {what}"));
        // Rust note: `map_err(|_| ...)` swallows the precise DomainError on
        // purpose — its message could echo stored values into logs.
        Ok(UserAccount::new(
            UserId::new(self.id).map_err(|_| corrupt("id"))?,
            EmailAddress::new(self.email).map_err(|_| corrupt("email"))?,
            CredentialHash::new(self.credential_hash).map_err(|_| corrupt("credential hash"))?,
            KdfParams::new(
                KdfAlgorithm::parse(&self.kdf_algorithm).map_err(|_| corrupt("kdf algorithm"))?,
                u32::try_from(self.kdf_memory_kib).map_err(|_| corrupt("kdf memory"))?,
                u32::try_from(self.kdf_iterations).map_err(|_| corrupt("kdf iterations"))?,
                u32::try_from(self.kdf_parallelism).map_err(|_| corrupt("kdf parallelism"))?,
            )
            .map_err(|_| corrupt("kdf params"))?,
            KeyBlob::new(self.wrapped_user_symmetric_key).map_err(|_| corrupt("wrapped usk"))?,
            KeyBlob::new(self.public_key).map_err(|_| corrupt("public key"))?,
            KeyBlob::new(self.wrapped_private_key).map_err(|_| corrupt("wrapped private key"))?,
        ))
    }
}

fn store_error(error: sqlx::Error) -> UserRepositoryError {
    UserRepositoryError::Store(error.to_string())
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn insert(&self, account: &UserAccount) -> Result<(), UserRepositoryError> {
        sqlx::query(
            "INSERT INTO users (id, email, credential_hash, kdf_algorithm, kdf_memory_kib, \
             kdf_iterations, kdf_parallelism, wrapped_user_symmetric_key, public_key, \
             wrapped_private_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account.id().as_str())
        .bind(account.email().as_str())
        .bind(account.credential_hash().as_str())
        .bind(account.kdf().algorithm().as_str())
        .bind(i64::from(account.kdf().memory_kib()))
        .bind(i64::from(account.kdf().iterations()))
        .bind(i64::from(account.kdf().parallelism()))
        .bind(account.wrapped_user_symmetric_key().as_bytes())
        .bind(account.public_key().as_bytes())
        .bind(account.wrapped_private_key().as_bytes())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            // Rust note: `matches!` + method chaining on Option — true only
            // when the driver says this was a UNIQUE constraint violation.
            if error
                .as_database_error()
                .is_some_and(|db| db.is_unique_violation())
            {
                UserRepositoryError::DuplicateEmail
            } else {
                store_error(error)
            }
        })?;
        Ok(())
    }

    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<UserAccount>, UserRepositoryError> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
            .bind(email.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(store_error)?;

        // Rust note: `Option::map` would trap the inner Result; `transpose`
        // flips Option<Result<..>> into Result<Option<..>> for `?`-friendliness.
        row.map(UserRow::into_account).transpose()
    }
}

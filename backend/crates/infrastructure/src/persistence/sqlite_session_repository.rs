use async_trait::async_trait;
use sqlx::SqlitePool;

use application::ports::{SessionRepository, SessionRepositoryError};
use domain::identity::{Session, UserId};

/// SQLite adapter for the [`SessionRepository`] port (refresh-token rotation
/// state). Pure storage — all decisions live in `Session::assess` and the
/// `RefreshSession` use case.
pub struct SqliteSessionRepository {
    pool: SqlitePool,
}

impl SqliteSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    family_id: String,
    user_id: String,
    token_hash: String,
    used: i64, // SQLite has no BOOLEAN; 0/1 by convention
    expires_at: i64,
}

impl SessionRow {
    fn into_session(self) -> Result<Session, SessionRepositoryError> {
        let corrupt = |what: &str| SessionRepositoryError(format!("corrupt session row: {what}"));
        Session::new(
            self.id,
            self.family_id,
            UserId::new(self.user_id).map_err(|_| corrupt("user id"))?,
            self.token_hash,
            self.used != 0,
            self.expires_at,
        )
        .map_err(|_| corrupt("fields"))
    }
}

fn store_error(error: sqlx::Error) -> SessionRepositoryError {
    SessionRepositoryError(error.to_string())
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn insert(&self, session: &Session) -> Result<(), SessionRepositoryError> {
        sqlx::query(
            "INSERT INTO refresh_sessions (id, family_id, user_id, token_hash, used, expires_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(session.id())
        .bind(session.family_id())
        .bind(session.user_id().as_str())
        .bind(session.token_hash())
        .bind(i64::from(session.used()))
        .bind(session.expires_at_unix())
        .execute(&self.pool)
        .await
        .map_err(store_error)?;
        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, SessionRepositoryError> {
        let row: Option<SessionRow> =
            sqlx::query_as("SELECT * FROM refresh_sessions WHERE token_hash = ?")
                .bind(token_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(store_error)?;
        row.map(SessionRow::into_session).transpose()
    }

    async fn mark_used(&self, session_id: &str) -> Result<(), SessionRepositoryError> {
        let result = sqlx::query("UPDATE refresh_sessions SET used = 1 WHERE id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(store_error)?;
        if result.rows_affected() == 0 {
            return Err(SessionRepositoryError("no such session".into()));
        }
        Ok(())
    }

    async fn revoke_family(&self, family_id: &str) -> Result<(), SessionRepositoryError> {
        sqlx::query("DELETE FROM refresh_sessions WHERE family_id = ?")
            .bind(family_id)
            .execute(&self.pool)
            .await
            .map_err(store_error)?;
        Ok(())
    }
}

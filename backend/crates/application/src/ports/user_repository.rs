use async_trait::async_trait;

use domain::identity::{EmailAddress, UserAccount};

/// Driven port: persistence for the [`UserAccount`] aggregate.
///
/// One repository per aggregate — however many tables the adapter needs,
/// this is the only doorway to user rows.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Persist a NEW account. Must fail with [`UserRepositoryError::DuplicateEmail`]
    /// if the email is already registered — the use case decides how (not
    /// whether) to hide that fact from the caller.
    async fn insert(&self, account: &UserAccount) -> Result<(), UserRepositoryError>;

    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<UserAccount>, UserRepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("email already registered")]
    DuplicateEmail,

    /// Anything infrastructural: connection lost, corrupt row, …
    #[error("user store failure: {0}")]
    Store(String),
}

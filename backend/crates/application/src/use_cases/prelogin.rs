use std::sync::Arc;

use domain::identity::{EmailAddress, KdfParams};

use crate::ports::UserRepository;

#[derive(Debug, thiserror::Error)]
#[error("prelogin failed: {0}")]
pub struct PreloginError(pub String);

/// Use case: hand the client the KDF parameters for an email so it can
/// derive its Master Key before login.
///
/// Anti-enumeration contract: unknown — and even syntactically invalid —
/// emails get the deterministic default parameters with the exact same
/// shape. The response never reveals whether an account exists.
pub struct Prelogin {
    users: Arc<dyn UserRepository>,
}

impl Prelogin {
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    pub async fn execute(&self, email: String) -> Result<KdfParams, PreloginError> {
        // A malformed email cannot belong to an account; answer with the
        // defaults rather than a validation error that leaks a difference.
        let Ok(email) = EmailAddress::new(email) else {
            return Ok(KdfParams::default_params());
        };

        let account = self
            .users
            .find_by_email(&email)
            .await
            .map_err(|error| PreloginError(error.to_string()))?;

        // Rust note: `map_or` collapses an Option — default for None,
        // closure result for Some — in one expression.
        Ok(account.map_or(KdfParams::default_params(), |a| a.kdf()))
    }
}

//! Identity context: user accounts, KDF parameters, wrapped user keys, and
//! rotating refresh-token sessions.
//!
//! Aggregate root: [`UserAccount`]. Sessions form their own small aggregate
//! ([`Session`]) — they come and go independently of the account.

mod credential_hash;
mod email_address;
mod kdf_params;
mod key_material;
mod session;
mod user_account;
mod user_id;

pub use credential_hash::CredentialHash;
pub use email_address::EmailAddress;
pub use kdf_params::{KdfAlgorithm, KdfParams};
pub use key_material::{KeyBlob, MasterPasswordHash};
pub use session::{Session, SessionAssessment};
pub use user_account::UserAccount;
pub use user_id::UserId;

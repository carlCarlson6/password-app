//! Use cases: one module per business operation. HTTP handlers call these
//! and nothing else.

pub mod check_health;
pub mod login;
pub mod prelogin;
pub mod refresh_session;
pub mod register_user;

pub use check_health::{CheckHealth, ComponentStatus, HealthReport};
pub use login::{LoggedIn, Login, LoginError, LoginInput};
pub use prelogin::{Prelogin, PreloginError};
pub use refresh_session::{RefreshSession, RefreshSessionError, RefreshedSession};
pub use register_user::{RegisterUser, RegisterUserError, RegisterUserInput};

//! Handlers + DTOs for `/api/auth/*`.
//!
//! Thin by rule: decode base64 / cookies at the edge, call a use case, map
//! the result. SECURITY: nothing in this module logs request bodies — they
//! carry credential hashes and wrapped keys.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};

use application::use_cases::{
    LoginError, LoginInput, RefreshSessionError, RegisterUserError, RegisterUserInput,
};
use domain::identity::KdfParams;

use crate::state::AppState;

const REFRESH_COOKIE: &str = "refresh_token";

// ---------- shared DTO pieces ----------

/// KDF parameters as the wire sees them (`{"algorithm": "argon2id",
/// "memoryKiB": ..., "iterations": ..., "parallelism": ...}`).
#[derive(Serialize, Deserialize)]
pub struct KdfDto {
    pub algorithm: String,
    // Rust note: serde attributes control the JSON name; "KiB" breaks the
    // usual camelCase convention, so it is pinned explicitly.
    #[serde(rename = "memoryKiB")]
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl From<KdfParams> for KdfDto {
    fn from(params: KdfParams) -> Self {
        Self {
            algorithm: params.algorithm().as_str().to_string(),
            memory_kib: params.memory_kib(),
            iterations: params.iterations(),
            parallelism: params.parallelism(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

fn error_response(status: StatusCode, message: &'static str) -> Response {
    (status, Json(ErrorBody { error: message })).into_response()
}

fn invalid_request() -> Response {
    error_response(StatusCode::BAD_REQUEST, "invalid request")
}

fn internal_error() -> Response {
    // Deliberately opaque; details went to the server log, not the client.
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
}

fn decode_base64(field: &str) -> Option<Vec<u8>> {
    BASE64.decode(field).ok()
}

// ---------- refresh cookie plumbing ----------

fn refresh_cookie(value: &str, max_age_seconds: i64, secure: bool) -> String {
    // httpOnly: JS can never read it. Path-scoped to /api/auth so it is not
    // sprayed on every API call. SameSite=Strict kills CSRF on the refresh
    // endpoint. `Secure` comes from config (off for plain-http local dev).
    format!(
        "{REFRESH_COOKIE}={value}; HttpOnly; SameSite=Strict; Path=/api/auth; Max-Age={max_age_seconds}{}",
        if secure { "; Secure" } else { "" }
    )
}

fn clear_refresh_cookie(secure: bool) -> String {
    refresh_cookie("", 0, secure)
}

fn extract_refresh_cookie(headers: &HeaderMap) -> Option<String> {
    // Rust note: `?` works on Option in functions returning Option — each
    // step short-circuits to None on failure.
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies
        .split(';')
        .map(str::trim)
        .find_map(|pair| pair.strip_prefix(&format!("{REFRESH_COOKIE}=")[..]))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

// ---------- POST /api/auth/prelogin ----------

#[derive(Deserialize)]
pub struct PreloginRequest {
    email: String,
}

#[derive(Serialize)]
pub struct PreloginResponse {
    kdf: KdfDto,
}

pub async fn prelogin(
    State(state): State<AppState>,
    Json(body): Json<PreloginRequest>,
) -> Response {
    match state.prelogin.execute(body.email).await {
        Ok(params) => (
            StatusCode::OK,
            Json(PreloginResponse { kdf: params.into() }),
        )
            .into_response(),
        Err(_) => internal_error(),
    }
}

// ---------- POST /api/auth/register ----------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    email: String,
    master_password_hash: String,
    kdf: KdfDto,
    wrapped_user_symmetric_key: String,
    public_key: String,
    wrapped_private_key: String,
}

#[derive(Serialize)]
pub struct EmptyResponse {}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Response {
    // Rust note: `let ... else` binds on Some and diverges (early-returns)
    // on None — undecodable base64 becomes the generic 400 right here.
    let Some(master_password_hash) = decode_base64(&body.master_password_hash) else {
        return invalid_request();
    };
    let Some(wrapped_user_symmetric_key) = decode_base64(&body.wrapped_user_symmetric_key) else {
        return invalid_request();
    };
    let Some(public_key) = decode_base64(&body.public_key) else {
        return invalid_request();
    };
    let Some(wrapped_private_key) = decode_base64(&body.wrapped_private_key) else {
        return invalid_request();
    };

    let input = RegisterUserInput {
        email: body.email,
        master_password_hash,
        kdf_algorithm: body.kdf.algorithm,
        kdf_memory_kib: body.kdf.memory_kib,
        kdf_iterations: body.kdf.iterations,
        kdf_parallelism: body.kdf.parallelism,
        wrapped_user_symmetric_key,
        public_key,
        wrapped_private_key,
    };

    match state.register_user.execute(input).await {
        // 201 whether the account was created or the email already existed —
        // the anti-enumeration contract (see README).
        Ok(()) => (StatusCode::CREATED, Json(EmptyResponse {})).into_response(),
        Err(RegisterUserError::Invalid(_)) => invalid_request(),
        Err(RegisterUserError::Infra(reason)) => {
            tracing::error!(%reason, "register failed");
            internal_error()
        }
    }
}

// ---------- POST /api/auth/login ----------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    email: String,
    master_password_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    access_token: String,
    wrapped_user_symmetric_key: String,
    public_key: String,
    wrapped_private_key: String,
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> Response {
    // Undecodable credential can never authenticate; same 401 as any
    // other bad credential, not a distinguishable 400.
    let Some(master_password_hash) = decode_base64(&body.master_password_hash) else {
        return error_response(StatusCode::UNAUTHORIZED, "invalid credentials");
    };

    let result = state
        .login
        .execute(LoginInput {
            email: body.email,
            master_password_hash,
        })
        .await;

    match result {
        Ok(logged_in) => {
            let cookie = refresh_cookie(
                &logged_in.refresh_token,
                logged_in.refresh_ttl_seconds,
                state.cookie_secure,
            );
            (
                StatusCode::OK,
                [(header::SET_COOKIE, cookie)],
                Json(LoginResponse {
                    access_token: logged_in.access_token,
                    wrapped_user_symmetric_key: BASE64.encode(logged_in.wrapped_user_symmetric_key),
                    public_key: BASE64.encode(logged_in.public_key),
                    wrapped_private_key: BASE64.encode(logged_in.wrapped_private_key),
                }),
            )
                .into_response()
        }
        Err(LoginError::InvalidCredentials) => {
            error_response(StatusCode::UNAUTHORIZED, "invalid credentials")
        }
        Err(LoginError::Infra(reason)) => {
            tracing::error!(%reason, "login failed");
            internal_error()
        }
    }
}

// ---------- POST /api/auth/refresh ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    access_token: String,
}

pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(presented) = extract_refresh_cookie(&headers) else {
        return error_response(StatusCode::UNAUTHORIZED, "invalid session");
    };

    match state.refresh_session.execute(&presented).await {
        Ok(refreshed) => {
            let cookie = refresh_cookie(
                &refreshed.refresh_token,
                refreshed.refresh_ttl_seconds,
                state.cookie_secure,
            );
            (
                StatusCode::OK,
                [(header::SET_COOKIE, cookie)],
                Json(RefreshResponse {
                    access_token: refreshed.access_token,
                }),
            )
                .into_response()
        }
        Err(RefreshSessionError::InvalidSession) => (
            StatusCode::UNAUTHORIZED,
            // Expire the useless cookie on the client too.
            [(
                header::SET_COOKIE,
                clear_refresh_cookie(state.cookie_secure),
            )],
            Json(ErrorBody {
                error: "invalid session",
            }),
        )
            .into_response(),
        Err(RefreshSessionError::Infra(reason)) => {
            tracing::error!(%reason, "refresh failed");
            internal_error()
        }
    }
}

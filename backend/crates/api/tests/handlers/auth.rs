//! Tests for `api/src/handlers/auth.rs`.
//!
//! End-to-end through the REAL stack: router → use cases → Argon2 hasher,
//! JWT issuer, SQLite (in-memory). Only the HTTP socket is skipped.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use api::middleware::RateLimitConfig;
use api::{AppState, build_router_with};
use application::use_cases::{CheckHealth, Login, Prelogin, RefreshSession, RegisterUser};
use infrastructure::persistence::{
    SqliteDatabaseProbe, SqliteSessionRepository, SqliteUserRepository, connect, run_migrations,
};
use infrastructure::security::{
    Argon2PasswordHasher, JwtTokenIssuer, Sha256RefreshTokenVendor, SystemClock, UuidGenerator,
};

const JWT_SECRET: &[u8] = b"test-secret";
const REFRESH_TTL: i64 = 1_209_600;

/// Real composition root against an in-memory database. Rate limits are
/// set sky-high — they have their own tests in `middleware/rate_limit.rs`.
async fn app(cookie_secure: bool) -> Router {
    let pool = connect("sqlite::memory:").await.expect("connect");
    run_migrations(&pool).await.expect("migrate");

    let users = Arc::new(SqliteUserRepository::new(pool.clone()));
    let sessions = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let hasher = Arc::new(Argon2PasswordHasher::new().expect("hasher"));
    let tokens = Arc::new(JwtTokenIssuer::new(JWT_SECRET, 900));
    let vendor = Arc::new(Sha256RefreshTokenVendor);
    let ids = Arc::new(UuidGenerator);
    let clock = Arc::new(SystemClock);

    let state = AppState {
        check_health: Arc::new(CheckHealth::new(Arc::new(SqliteDatabaseProbe::new(pool)))),
        register_user: Arc::new(RegisterUser::new(
            users.clone(),
            hasher.clone(),
            ids.clone(),
        )),
        prelogin: Arc::new(Prelogin::new(users.clone())),
        login: Arc::new(Login::new(
            users,
            sessions.clone(),
            hasher,
            tokens.clone(),
            vendor.clone(),
            ids.clone(),
            clock.clone(),
            REFRESH_TTL,
        )),
        refresh_session: Arc::new(RefreshSession::new(
            sessions,
            tokens,
            vendor,
            ids,
            clock,
            REFRESH_TTL,
        )),
        cookie_secure,
    };

    build_router_with(
        state,
        RateLimitConfig {
            max_requests: 10_000,
            ..RateLimitConfig::default()
        },
    )
}

/// POST a JSON body (optionally with a Cookie header) and split the response
/// into status, Set-Cookie (if any) and parsed JSON body.
async fn post(
    app: &Router,
    path: &str,
    body: Value,
    cookie: Option<&str>,
) -> (StatusCode, Option<String>, Value) {
    let mut request = Request::post(path).header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).expect("request"))
        .await
        .expect("infallible router");

    let status = response.status();
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .map(|v| v.to_str().expect("ascii cookie").to_string());
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("json body")
    };
    (status, set_cookie, body)
}

fn register_body(email: &str) -> Value {
    json!({
        "email": email,
        // base64 of 32 bytes of 0x07
        "masterPasswordHash": "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        "kdf": {"algorithm": "argon2id", "memoryKiB": 65536, "iterations": 3, "parallelism": 4},
        "wrappedUserSymmetricKey": "d3JhcHBlZC11c2s=",   // "wrapped-usk"
        "publicKey": "cHVibGljLWtleQ==",                  // "public-key"
        "wrappedPrivateKey": "d3JhcHBlZC1wcml2YXRl"       // "wrapped-private"
    })
}

fn login_body(email: &str, mph_b64: &str) -> Value {
    json!({ "email": email, "masterPasswordHash": mph_b64 })
}

/// Pull the `refresh_token=...` value out of a Set-Cookie header.
fn cookie_value(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string()
}

// ---------- register ----------

#[tokio::test]
async fn register_returns_created_and_duplicates_are_indistinguishable() {
    let app = app(false).await;

    let (status, _, body) = post(
        &app,
        "/api/auth/register",
        register_body("a@example.com"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, json!({}));

    // Same email, different credential: byte-identical success response.
    let mut retry = register_body("a@example.com");
    retry["masterPasswordHash"] = json!("CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk=");
    let (status, _, body) = post(&app, "/api/auth/register", retry, None).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, json!({}));
}

#[tokio::test]
async fn register_rejects_bad_base64_and_bad_kdf_params() {
    let app = app(false).await;

    let mut bad_b64 = register_body("a@example.com");
    bad_b64["masterPasswordHash"] = json!("!!! not base64 !!!");
    let (status, _, _) = post(&app, "/api/auth/register", bad_b64, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let mut weak_kdf = register_body("a@example.com");
    weak_kdf["kdf"]["memoryKiB"] = json!(16);
    let (status, _, _) = post(&app, "/api/auth/register", weak_kdf, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------- prelogin ----------

#[tokio::test]
async fn prelogin_answers_identically_for_known_and_unknown_emails() {
    let app = app(false).await;
    post(
        &app,
        "/api/auth/register",
        register_body("known@example.com"),
        None,
    )
    .await;

    let (status_known, _, known) = post(
        &app,
        "/api/auth/prelogin",
        json!({"email": "known@example.com"}),
        None,
    )
    .await;
    let (status_unknown, _, unknown) = post(
        &app,
        "/api/auth/prelogin",
        json!({"email": "ghost@example.com"}),
        None,
    )
    .await;
    let (status_malformed, _, malformed) = post(
        &app,
        "/api/auth/prelogin",
        json!({"email": "not-an-email"}),
        None,
    )
    .await;

    assert_eq!(status_known, StatusCode::OK);
    assert_eq!(status_unknown, StatusCode::OK);
    assert_eq!(status_malformed, StatusCode::OK);

    // The exact contract shape, and zero difference across the three cases
    // (the account above registered with the default params).
    let expected = json!({
        "kdf": {"algorithm": "argon2id", "memoryKiB": 65536, "iterations": 3, "parallelism": 4}
    });
    assert_eq!(known, expected);
    assert_eq!(unknown, expected);
    assert_eq!(malformed, expected);
}

#[tokio::test]
async fn prelogin_serves_the_users_stored_params_when_they_differ() {
    let app = app(false).await;
    let mut custom = register_body("custom@example.com");
    custom["kdf"] =
        json!({"algorithm": "argon2id", "memoryKiB": 32768, "iterations": 5, "parallelism": 2});
    post(&app, "/api/auth/register", custom, None).await;

    let (_, _, body) = post(
        &app,
        "/api/auth/prelogin",
        json!({"email": "custom@example.com"}),
        None,
    )
    .await;
    assert_eq!(body["kdf"]["memoryKiB"], 32768);
    assert_eq!(body["kdf"]["iterations"], 5);
    assert_eq!(body["kdf"]["parallelism"], 2);
}

// ---------- login ----------

#[tokio::test]
async fn login_returns_tokens_wrapped_keys_and_an_http_only_refresh_cookie() {
    let app = app(false).await;
    post(
        &app,
        "/api/auth/register",
        register_body("a@example.com"),
        None,
    )
    .await;

    let (status, set_cookie, body) = post(
        &app,
        "/api/auth/login",
        login_body(
            "a@example.com",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        ),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // The wrapped blobs come back exactly as registered (base64, camelCase).
    assert_eq!(body["wrappedUserSymmetricKey"], "d3JhcHBlZC11c2s=");
    assert_eq!(body["publicKey"], "cHVibGljLWtleQ==");
    assert_eq!(body["wrappedPrivateKey"], "d3JhcHBlZC1wcml2YXRl");

    // The access token is a real, verifiable JWT for a real user id.
    let claims = JwtTokenIssuer::new(JWT_SECRET, 900)
        .verify(body["accessToken"].as_str().expect("accessToken"))
        .expect("valid jwt");
    assert!(!claims.sub.is_empty());
    assert_eq!(claims.exp, claims.iat + 900);

    // Refresh token rides in a hardened, path-scoped, httpOnly cookie.
    let cookie = set_cookie.expect("Set-Cookie present");
    assert!(cookie.starts_with("refresh_token="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Path=/api/auth"));
    assert!(cookie.contains(&format!("Max-Age={REFRESH_TTL}")));
    assert!(!cookie.contains("Secure")); // cookie_secure=false (dev)
    // And it is NOT the access token, nor anywhere in the body.
    let raw = cookie_value(&cookie);
    assert!(
        !body
            .to_string()
            .contains(raw.trim_start_matches("refresh_token="))
    );
}

#[tokio::test]
async fn login_cookie_is_secure_when_configured() {
    let app = app(true).await;
    post(
        &app,
        "/api/auth/register",
        register_body("a@example.com"),
        None,
    )
    .await;
    let (_, set_cookie, _) = post(
        &app,
        "/api/auth/login",
        login_body(
            "a@example.com",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        ),
        None,
    )
    .await;
    assert!(set_cookie.expect("cookie").contains("; Secure"));
}

#[tokio::test]
async fn login_failure_is_identical_for_wrong_password_and_unknown_email() {
    let app = app(false).await;
    post(
        &app,
        "/api/auth/register",
        register_body("a@example.com"),
        None,
    )
    .await;

    let (status_wrong, cookie_wrong, body_wrong) = post(
        &app,
        "/api/auth/login",
        login_body(
            "a@example.com",
            "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk=",
        ),
        None,
    )
    .await;
    let (status_ghost, cookie_ghost, body_ghost) = post(
        &app,
        "/api/auth/login",
        login_body(
            "ghost@example.com",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        ),
        None,
    )
    .await;

    assert_eq!(status_wrong, StatusCode::UNAUTHORIZED);
    assert_eq!(status_ghost, StatusCode::UNAUTHORIZED);
    assert_eq!(body_wrong, body_ghost); // byte-identical error
    assert!(cookie_wrong.is_none());
    assert!(cookie_ghost.is_none());
}

// ---------- refresh ----------

#[tokio::test]
async fn refresh_rotates_the_cookie_and_replay_kills_the_family() {
    let app = app(false).await;
    post(
        &app,
        "/api/auth/register",
        register_body("a@example.com"),
        None,
    )
    .await;
    let (_, first_cookie, _) = post(
        &app,
        "/api/auth/login",
        login_body(
            "a@example.com",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        ),
        None,
    )
    .await;
    let first = cookie_value(&first_cookie.expect("login cookie"));

    // Legitimate refresh: new access token + a DIFFERENT cookie.
    let (status, second_cookie, body) =
        post(&app, "/api/auth/refresh", json!({}), Some(&first)).await;
    assert_eq!(status, StatusCode::OK);
    let second = cookie_value(&second_cookie.expect("rotated cookie"));
    assert_ne!(first, second);
    assert!(
        JwtTokenIssuer::new(JWT_SECRET, 900)
            .verify(body["accessToken"].as_str().expect("accessToken"))
            .is_ok()
    );

    // Replaying the rotated-out cookie: rejected AND the family dies.
    let (status, cleared, _) = post(&app, "/api/auth/refresh", json!({}), Some(&first)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(cleared.expect("clearing cookie").contains("Max-Age=0"));

    // Even the "legitimate" successor cookie is now dead.
    let (status, _, _) = post(&app, "/api/auth/refresh", json!({}), Some(&second)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_without_or_with_a_bogus_cookie_is_unauthorized() {
    let app = app(false).await;

    let (status, _, _) = post(&app, "/api/auth/refresh", json!({}), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _, _) = post(
        &app,
        "/api/auth/refresh",
        json!({}),
        Some("refresh_token=never-issued"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// Keep the connect-info import "used" even though oneshot tests skip it;
// documents that production keys rate limits by SocketAddr.
#[allow(dead_code)]
fn _socket_addr_used_in_main(_: SocketAddr) {}

//! Tests for `application/src/use_cases/refresh_session.rs`.

use std::sync::Arc;

use application::use_cases::{RefreshSession, RefreshSessionError};
use domain::identity::{Session, UserId};

use crate::support::{FakeTokens, FakeVendor, FixedClock, InMemorySessions, SeqIds};

const REFRESH_TTL: i64 = 1_209_600;

struct World {
    sessions: Arc<InMemorySessions>,
    clock: Arc<FixedClock>,
    refresh: RefreshSession,
}

/// One live session, as a login would have left it: token "raw-0" (hashed),
/// family "family-1", expiring at 1_000 + TTL. The vendor's next mint is "raw-1".
fn world() -> World {
    let sessions = Arc::new(InMemorySessions::default());
    let clock = Arc::new(FixedClock::at(1_000));
    sessions.add(
        Session::new(
            "session-1",
            "family-1",
            UserId::new("user-1").unwrap(),
            "hashed:raw-0",
            false,
            1_000 + REFRESH_TTL,
        )
        .unwrap(),
    );
    let refresh = RefreshSession::new(
        sessions.clone(),
        Arc::new(FakeTokens),
        Arc::new(FakeVendor::default()),
        Arc::new(SeqIds::default()),
        clock.clone(),
        REFRESH_TTL,
    );
    World {
        sessions,
        clock,
        refresh,
    }
}

#[tokio::test]
async fn rotates_the_token_and_issues_a_new_access_token() {
    let world = world();
    let refreshed = world.refresh.execute("raw-0").await.unwrap();

    assert_eq!(refreshed.access_token, "access:user-1:1000");
    assert_eq!(refreshed.refresh_token, "raw-1");
    assert_eq!(refreshed.refresh_ttl_seconds, REFRESH_TTL);

    let sessions = world.sessions.all();
    assert_eq!(sessions.len(), 2);
    // The presented token is retired but KEPT (to catch replays)…
    assert!(sessions[0].used());
    // …and its successor joined the same family, unused.
    assert_eq!(sessions[1].family_id(), "family-1");
    assert_eq!(sessions[1].token_hash(), "hashed:raw-1");
    assert!(!sessions[1].used());
    assert_eq!(sessions[1].user_id().as_str(), "user-1");
}

#[tokio::test]
async fn an_unknown_token_is_invalid() {
    let world = world();
    assert!(matches!(
        world.refresh.execute("never-issued").await,
        Err(RefreshSessionError::InvalidSession)
    ));
}

#[tokio::test]
async fn an_expired_token_is_invalid() {
    let world = world();
    world.clock.set(1_000 + REFRESH_TTL); // exactly at expiry
    assert!(matches!(
        world.refresh.execute("raw-0").await,
        Err(RefreshSessionError::InvalidSession)
    ));
}

#[tokio::test]
async fn replaying_a_rotated_out_token_revokes_the_whole_family() {
    let world = world();

    // Legitimate rotation: raw-0 → raw-1.
    world.refresh.execute("raw-0").await.unwrap();
    assert_eq!(world.sessions.all().len(), 2);

    // Replay of the retired raw-0: theft assumed.
    assert!(matches!(
        world.refresh.execute("raw-0").await,
        Err(RefreshSessionError::InvalidSession)
    ));
    // The ENTIRE family is gone — including the fresh raw-1…
    assert!(world.sessions.all().is_empty());
    // …so even the "legitimate" successor is now useless.
    assert!(matches!(
        world.refresh.execute("raw-1").await,
        Err(RefreshSessionError::InvalidSession)
    ));
}

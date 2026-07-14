-- Phase 1: Identity context — user accounts and rotating refresh sessions.
--
-- Zero-knowledge invariants, enforced by shape:
--   * credential_hash is the server-side Argon2id RE-hash (PHC string);
--     the client-sent login hash is never persisted.
--   * The three key columns are opaque ciphertext blobs the client produced.
--     Nothing in this schema can name or reveal vault contents.

CREATE TABLE users (
    id                         TEXT    PRIMARY KEY,
    email                      TEXT    NOT NULL UNIQUE,
    credential_hash            TEXT    NOT NULL,
    kdf_algorithm              TEXT    NOT NULL,
    kdf_memory_kib             INTEGER NOT NULL,
    kdf_iterations             INTEGER NOT NULL,
    kdf_parallelism            INTEGER NOT NULL,
    wrapped_user_symmetric_key BLOB    NOT NULL,
    public_key                 BLOB    NOT NULL,
    wrapped_private_key        BLOB    NOT NULL
) STRICT;

-- Rotating refresh tokens. Only the SHA-256 hash of a token is stored, so a
-- database leak cannot forge cookies. `used` rows are kept until family
-- revocation/expiry so replays of rotated-out tokens are detectable.
CREATE TABLE refresh_sessions (
    id         TEXT    PRIMARY KEY,
    family_id  TEXT    NOT NULL,
    user_id    TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT    NOT NULL UNIQUE,
    used       INTEGER NOT NULL DEFAULT 0,
    expires_at INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_refresh_sessions_family_id ON refresh_sessions(family_id);

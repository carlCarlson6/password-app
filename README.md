# Password App (working title)

A zero-knowledge, multi-user password manager. Rust backend, React frontend,
clean DDD + hexagonal architecture on both sides.

**Status:** Phase 0 in progress — the walking skeleton runs end to end
(UI → Axum → use case → SQLite); Docker Compose + CI still pending.

---

## 1. Decisions record

| Decision | Choice | Rationale / notes |
|---|---|---|
| Deployment | Public cloud service (multi-tenant) | SaaS-style; hardening, rate limiting and abuse controls are in scope. |
| Encryption model | **Zero-knowledge / E2E** | All vault data is encrypted client-side. The server stores only ciphertext and can never read secrets. |
| Users | Multi-user, isolated vaults | A user can own **multiple vaults**. Design must enable **shared vaults later** without a data migration (see key hierarchy). |
| Auth | Bitwarden-style derived hash | Client derives login credential from the master password via Argon2id; server never sees the master password. |
| Recovery | **Deferred to v2** ⚠️ | Forgotten master password = unrecoverable vault at launch. The key hierarchy reserves a slot for recovery keys so adding them later requires no re-encryption. This must be communicated loudly in the signup UX. |
| Database | SQLite (via SQLx) | Zero-ops to start; Litestream for replication/backup. Hexagonal ports make a Postgres adapter a drop-in later when multi-tenant load demands it. |
| Backend framework | Axum | Tokio-native, tower middleware, type-safe extractors. |
| Frontend | React + TypeScript + Vite | TanStack Query for server state, Zustand for client state, Tailwind for styling. (Defaults — cheap to swap.) |
| V1 features | Vault CRUD, auth, crypto core, **password generator**, **folders/tags/search** | TOTP storage, import/export, recovery keys, sharing → v2 roadmap. |

---

## 2. Security model

### Zero-knowledge principle

The server is an untrusted ciphertext store. Everything the server persists
about vault contents is opaque bytes. All key derivation, encryption and
decryption happen in the browser (WebCrypto + a WASM Argon2id).

### Key hierarchy (client-side)

```
Master Password + email (salt)
        │  Argon2id (m=64MiB, t=3, p=4)
        ▼
   Master Key ──────────────► Argon2id(MK, password) = Master Password Hash
        │  HKDF-SHA256              │ sent to server as the LOGIN credential;
        ▼                           │ server re-hashes it (Argon2id) before storing
 Stretched Master Key
        │  wraps (AES-256-GCM)
        ▼
 User Symmetric Key  (random 256-bit, generated at signup)
        │                                   │
        │ wraps                             │ wraps
        ▼                                   ▼
 User Keypair (RSA-OAEP-2048,         Vault Key (random, one per vault)
 private key encrypted with USK)            │ encrypts (AES-256-GCM)
                                            ▼
                                      Vault items (ciphertext blobs)
```

Why this shape:

- **Master password change** re-wraps only the User Symmetric Key — no data re-encryption.
- **Sharing later:** to share a vault, wrap its Vault Key with the recipient's
  public key. No item is ever re-encrypted; the server just stores another
  wrapped copy of the Vault Key. The keypair is generated at signup precisely
  so sharing can ship in v2 with zero migration.
- **Recovery later (v2):** a recovery key is just one more wrapping of the
  User Symmetric Key. Adding it never touches vault data.

### Authentication flow

1. Client fetches KDF parameters (salt, Argon2id settings) for the email.
2. Client derives Master Key → Master Password Hash, sends it as the credential.
3. Server verifies against its stored Argon2id(MPH) and issues a short-lived
   access token (JWT) + rotating refresh token.
4. Server returns the user's wrapped keys; client unwraps them locally.

The server never sees the master password or any unwrapped key. TLS everywhere;
KDF-parameter responses for unknown emails are faked to prevent enumeration.

### Server-side hardening (public service)

- Argon2id re-hash of the login credential at rest.
- Rate limiting + exponential backoff on auth endpoints (tower middleware).
- Constant-time comparisons, no user-enumeration in errors or timings.
- Strict CORS, CSP, security headers; refresh tokens in httpOnly cookies.
- Audit log of auth events (metadata only — never vault content).

---

## 3. Architecture

### DDD view

Two domains exist, and they are **not** the same domain:

- **Server domain — "Vault Custody":** accounts, ownership, access control,
  wrapped keys, ciphertext lifecycle, sync. The server's `VaultItem` has no
  notion of "password" or "username" — its content is a `CipherBlob` value
  object. This is the honest model for zero-knowledge: the server's ubiquitous
  language is about custody, not credentials.
- **Client domain — "Credentials":** the decrypted world. Login entries,
  secure notes, folders, tags, the generator's policy rules, search. This
  domain lives entirely in the browser.

Server bounded contexts:

| Context | Aggregates | Responsibility |
|---|---|---|
| Identity | `UserAccount` | Registration, login credential verification, KDF params, sessions, wrapped user keys |
| Vaulting | `Vault` (root) → `VaultItem` | Vault lifecycle, item ciphertext CRUD, versioning/optimistic concurrency |
| Access (v2-ready) | `VaultGrant` | Who holds a wrapped copy of which Vault Key, with what role. In v1 there is exactly one grant per vault (the owner) — the aggregate exists from day one so sharing is additive. |

### Hexagonal layout — backend (Cargo workspace)

```
backend/
├── Cargo.toml                    # workspace
└── crates/
    ├── domain/                   # pure: entities, VOs, domain errors. No async, no deps on anything below.
    │   └── src/{identity,vaulting,access}/
    ├── application/              # use cases (driving ports) + port traits (driven ports)
    │   └── src/
    │       ├── use_cases/        # RegisterUser, Login, CreateVault, PutItem, ...
    │       └── ports/            # UserRepository, VaultRepository, Clock, TokenIssuer, PasswordHasher
    ├── infrastructure/           # driven adapters
    │   └── src/
    │       ├── persistence/      # SQLx + SQLite repositories, migrations
    │       ├── security/         # argon2 hasher, jwt issuer
    │       └── config/
    └── api/                      # driving adapter: Axum handlers, DTOs, middleware, main.rs
```

Dependency rule: `api → application → domain` and
`infrastructure → application (ports) + domain`. The domain crate compiles
with no async runtime and no I/O — it is unit-testable in microseconds.

Tests never live inline in source files: each crate keeps them in its
`tests/` directory, mirroring the `src/` module tree, with `tests/main.rs`
as the single harness entry point (see CLAUDE.md, "Testing style"). The
frontend follows the same convention (`frontend/tests/` mirrors `src/`).

### Hexagonal layout — frontend

```
frontend/
└── src/
    ├── domain/           # decrypted models: LoginEntry, Folder, Tag; generator rules; pure TS
    ├── application/      # use cases as hooks/services: unlockVault, saveEntry, search
    ├── infrastructure/   # adapters: API client, WebCrypto/Argon2-WASM CryptoService,
    │                     #   in-memory key store (keys NEVER hit localStorage)
    └── ui/               # routes, components, design system
```

The `CryptoService` is a port: the UI and use cases never call WebCrypto
directly, which keeps the crypto swappable (e.g., OPAQUE later) and testable.

### API sketch (v1)

```
POST   /api/auth/prelogin          → KDF params for email
POST   /api/auth/register
POST   /api/auth/login             → tokens + wrapped keys
POST   /api/auth/refresh
GET    /api/vaults                 → vaults + this user's wrapped vault keys
POST   /api/vaults
PUT    /api/vaults/:id             /  DELETE
GET    /api/vaults/:id/items       → ciphertext blobs (+ version stamps)
POST   /api/vaults/:id/items
PUT    /api/vaults/:id/items/:id   # optimistic concurrency via item version
DELETE /api/vaults/:id/items/:id
```

Folders/tags/search are **client-side concerns** encoded inside item
ciphertext (plus an encrypted per-vault folder tree blob) — the server never
learns the organization structure either.

---

## 4. Build plan

### Phase 0 — Scaffolding (the walking skeleton)
- [x] Cargo workspace with the four crates; empty ports wired end to end
- [x] Vite + React + TS app with the four-layer folder structure
- [x] SQLx + SQLite migrations setup; config loading; error taxonomy
- [ ] Docker Compose for local dev; CI: `cargo test/clippy/fmt`, `tsc`, `vitest`
- **Done when:** one dummy request travels UI → Axum → use case → SQLite and back.
  ✅ verified: `GET /api/health` through the Vite proxy returns
  `{"status":"ok","database":"up"}` from a live SQLite pool.

### Phase 1 — Crypto core & identity
- [ ] Frontend `CryptoService`: Argon2id (WASM), HKDF, AES-256-GCM, RSA keypair gen; test vectors pinned
- [ ] Register: derive keys client-side, ship wrapped keys + login hash
- [ ] Login/prelogin/refresh; Argon2 re-hash server-side; rate limiting
- [ ] Unlock flow: keys held in memory only; auto-lock on idle/tab close
- [ ] Signup UX warns explicitly: no recovery in v1
- **Done when:** a registered user can log in from a fresh browser and unwrap their keys; server DB provably contains no plaintext.

### Phase 2 — Vaults & items
- [ ] `Vault`, `VaultItem`, `VaultGrant` aggregates + SQLite repositories
- [ ] Vault CRUD with per-vault keys wrapped under the User Symmetric Key
- [ ] Item CRUD with versioning/optimistic concurrency
- [ ] Frontend: vault list, item list/detail/edit, encrypt-on-save decrypt-on-load
- **Done when:** full round-trip of a credential through two devices/browsers.

### Phase 3 — V1 features
- [ ] Password generator (client-only): length, charsets, passphrase mode, strength meter
- [ ] Folders & tags (encrypted folder tree blob), client-side fuzzy search
- [ ] Copy-to-clipboard with auto-clear; reveal toggles
- **Done when:** the app is daily-drivable for one person.

### Phase 4 — Public-service hardening & deploy
- [ ] Security headers, CSP, CORS lockdown; dependency audit (`cargo audit`, `npm audit`)
- [ ] Abuse controls: per-IP and per-account rate limits, signup throttling
- [ ] Litestream backup/replication for SQLite; health checks; structured logging (no secrets)
- [ ] Deploy target (single-node: Fly.io / Hetzner + Docker); TLS
- [ ] Threat-model pass and external review of the crypto flows before real users
- **Done when:** deployed, backed up, monitored, and reviewed.

### v2 roadmap (in priority order)
1. **Recovery keys** — one-time code wrapping the User Symmetric Key (highest priority: launch risk until done)
2. **Shared vaults** — wrap Vault Keys to recipients' public keys; roles on `VaultGrant`
3. TOTP storage, import/export, Postgres adapter, browser extension, WebAuthn 2FA

---

## 5. Development

```sh
# backend (from backend/)
cargo test                                # tests live in each crate's tests/, mirroring src/
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo run -p api                          # http://127.0.0.1:8080, SQLite at data/app.db

# frontend (from frontend/)
npm install
npm run dev                               # http://localhost:5173, proxies /api → 127.0.0.1:8080
npm run typecheck
npm test -- --run
npm run build
```

Backend configuration comes from the environment, with dev defaults:
`DATABASE_URL` (`sqlite://data/app.db` — the `data/` directory must exist;
migrations run automatically at startup) and `BIND_ADDR` (`127.0.0.1:8080`).

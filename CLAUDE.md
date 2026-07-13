# Password App — project instructions

Zero-knowledge, multi-user password manager. Rust (Axum) backend, React (Vite + TS)
frontend, clean DDD + hexagonal architecture on both sides. `README.md` is the
architectural record and phased build plan — read it before making design decisions.

## Keep documentation up to date (always)

Every change that affects architecture, the security/key model, API shapes, project
structure, commands, or phase progress MUST update the documentation in the same
change:

- `README.md` — tick/untick phase checkboxes, update the decisions record, API
  sketch, and Development section when they drift from reality.
- `CLAUDE.md` — update when conventions or workflows change.

Docs describing the system wrongly are worse than no docs. Never leave this for later.

## Worktree workflow

- When an implementation or set of changes is finished in a worktree, **do not merge
  it into `main` on your own**. Stop, summarize what changed, and wait for the
  project owner's explicit approval before merging.

## Architecture rules

- Dependency rule: `api → application → domain`; `infrastructure` implements
  `application` ports. The `domain` crate has **no async, no I/O, no framework deps**.
- The server never sees plaintext secrets or unwrapped keys. Server-side `VaultItem`
  content is an opaque ciphertext blob — never add fields that reveal vault contents
  or structure.
- New behavior goes through a use case in `application/`; HTTP handlers stay thin
  (deserialize → call use case → map to response). No handler talks to SQLx directly.
- One repository per aggregate, not per table. Frontend mirrors the same layering:
  `domain/` (pure), `application/` (use cases + ports), `infrastructure/` (fetch,
  WebCrypto), `ui/` (React). UI components never call fetch or WebCrypto directly.
- Vault keys and derived keys live in memory only — never in localStorage/sessionStorage.

## Testing style

- Every new feature MUST ship with tests that verify its behavior, in the same
  change — new use cases, domain logic, and endpoints are not done until tests
  prove the implementation is correct. This applies to both backend and frontend.
- Backend tests never live inline in source files (`#[cfg(test)] mod tests` is
  banned). Each crate keeps all its tests in its `tests/` directory, mirroring
  the `src/` folder structure: the test for `src/identity/email_address.rs` is
  `tests/identity/email_address.rs`.
- Each crate's `tests/main.rs` is the single harness entry point; it (and
  `mod.rs` files in subfolders) declare the mirrored module tree, because Cargo
  only auto-compiles top-level files in `tests/`.
- Consequence to embrace: tests exercise the crate's **public API only**. If
  something can't be tested that way, reconsider its visibility or design
  rather than moving the test inline.

## Code style

- The project owner is learning Rust: annotate Rust code with brief `// Rust note:`
  comments explaining language concepts (ownership, traits, `?`, lifetimes, async)
  the first time they appear in a file. Keep them short and educational, not noisy.

## Commands

- Dev stack (from repo root): `./scripts/dev.sh` starts backend + frontend (+ any
  future infra) via Docker Compose with hot reload; `detach|logs|down` subcommands.
  SQLite is a bind-mounted file (`backend/data/app.db`), not a compose service.
- Local CI (from repo root): `./scripts/ci.sh` runs the full pipeline below;
  `./scripts/ci.sh backend|frontend` runs one side only.
- Backend (from `backend/`): `cargo test`, `cargo clippy --workspace -- -D warnings`,
  `cargo fmt --all`, `cargo run -p api`
- Frontend (from `frontend/`): `npm run dev`, `npm run typecheck`, `npm test -- --run`,
  `npm run build`

Both must pass fmt/clippy/typecheck/tests before a change is considered done —
`./scripts/ci.sh` checks all of it in one go.

#!/usr/bin/env bash
#
# Local CI pipeline — runs the same checks a change must pass before it is
# considered done (see CLAUDE.md): fmt, clippy, build and tests for the
# backend; typecheck, tests and build for the frontend.
#
# Usage:
#   ./scripts/ci.sh             # run everything
#   ./scripts/ci.sh backend     # backend checks only
#   ./scripts/ci.sh frontend    # frontend checks only

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-all}"

BOLD=$'\033[1m'
GREEN=$'\033[32m'
RED=$'\033[31m'
RESET=$'\033[0m'

PASSED_STEPS=()

step() {
    local name="$1"
    shift
    echo
    echo "${BOLD}==> ${name}${RESET}"
    local start
    start=$(date +%s)
    if "$@"; then
        local elapsed=$(( $(date +%s) - start ))
        echo "${GREEN}✔ ${name} (${elapsed}s)${RESET}"
        PASSED_STEPS+=("${name}")
    else
        echo "${RED}✘ ${name} failed${RESET}" >&2
        exit 1
    fi
}

run_backend() {
    cd "${REPO_ROOT}/backend"
    step "backend: cargo fmt (check)"  cargo fmt --all -- --check
    step "backend: cargo clippy"       cargo clippy --workspace -- -D warnings
    step "backend: cargo build"        cargo build --workspace
    step "backend: cargo test"         cargo test --workspace
}

run_frontend() {
    cd "${REPO_ROOT}/frontend"
    if [ ! -d node_modules ]; then
        step "frontend: npm ci" npm ci
    fi
    step "frontend: typecheck"  npm run typecheck
    step "frontend: tests"      npm test -- --run
    step "frontend: build"      npm run build
}

case "${TARGET}" in
    all)      run_backend; run_frontend ;;
    backend)  run_backend ;;
    frontend) run_frontend ;;
    *)
        echo "usage: $0 [all|backend|frontend]" >&2
        exit 2
        ;;
esac

echo
echo "${GREEN}${BOLD}CI passed — ${#PASSED_STEPS[@]} step(s) green.${RESET}"

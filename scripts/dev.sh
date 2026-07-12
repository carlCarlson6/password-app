#!/usr/bin/env bash
#
# Local dev stack — builds (when needed) and starts everything defined in
# docker-compose.yml:
#
#   backend    Axum API with hot reload (cargo-watch)   http://localhost:8080
#   frontend   Vite dev server with HMR                 http://localhost:5173
#   database   none — SQLite is a file (backend/data/app.db), not a service
#
# Usage:
#   ./scripts/dev.sh              # start in the foreground; Ctrl-C stops all
#   ./scripts/dev.sh detach       # start in the background
#   ./scripts/dev.sh logs         # follow logs of a detached stack
#   ./scripts/dev.sh down         # stop and remove the containers
#   ./scripts/dev.sh <cmd> [...]  # anything else is passed to `docker compose`

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${REPO_ROOT}/docker-compose.yml"

RED=$'\033[31m'
RESET=$'\033[0m'

die() {
    echo "${RED}error:${RESET} $*" >&2
    exit 1
}

command -v docker >/dev/null 2>&1 \
    || die "docker is not installed — get it from https://docs.docker.com/get-docker/"
docker info >/dev/null 2>&1 \
    || die "the docker daemon is not running — start Docker Desktop and retry"
docker compose version >/dev/null 2>&1 \
    || die "the docker compose plugin is missing (Compose v2 is required)"
[ -f "${COMPOSE_FILE}" ] \
    || die "docker-compose.yml not found at the repo root (${REPO_ROOT})"

# The backend expects data/ to exist (SQLite lives there). It is .gitkeep'd,
# but recreate it defensively for clones that lost it.
mkdir -p "${REPO_ROOT}/backend/data"

cd "${REPO_ROOT}"

CMD="${1:-up}"
if [ "$#" -gt 0 ]; then
    shift
fi

case "${CMD}" in
    up)     exec docker compose up --build "$@" ;;
    detach) exec docker compose up --build --detach "$@" ;;
    logs)   exec docker compose logs --follow "$@" ;;
    *)      exec docker compose "${CMD}" "$@" ;;
esac

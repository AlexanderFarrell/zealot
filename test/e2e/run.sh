#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
VENV_DIR="$SCRIPT_DIR/.venv"
PYTHON_BIN="$VENV_DIR/bin/python"
STAMP_FILE="$VENV_DIR/.requirements-installed"
REQUIREMENTS_FILE="$SCRIPT_DIR/requirements.txt"
COMPOSE_FILE="$SCRIPT_DIR/compose.yml"
SERVER_URL="http://127.0.0.1:18456"
WEB_URL="http://127.0.0.1:18080"
POSTGRES_DSN="postgresql://zealot:zealot@127.0.0.1:15432/zealot"

compose() {
    docker compose -f "$COMPOSE_FILE" "$@"
}

wait_for_http() {
    url=$1
    expected_body=${2-}
    attempts=0

    while [ "$attempts" -lt 90 ]; do
        response=$(curl -fsS "$url" 2>/dev/null || true)
        if [ -n "$response" ]; then
            if [ -z "$expected_body" ] || [ "$response" = "$expected_body" ]; then
                echo "" >&2
                return 0
            fi
        fi
        attempts=$((attempts + 1))
        printf "." >&2
        sleep 2
    done

    echo "" >&2
    echo "[e2e] Timed out waiting for $url" >&2
    return 1
}

cleanup() {
    echo "[e2e] Stopping docker compose stack"
    compose --profile postgres down -v --remove-orphans
}

run_backend() {
    database=$1
    shift
    compose --profile postgres down -v --remove-orphans

    if [ "$database" = "postgres" ]; then
        echo "[e2e] Starting PostgreSQL-backed stack"
        if ! compose --profile postgres up --wait --remove-orphans postgres; then
            echo "[e2e] PostgreSQL stack could not be started" >&2
            return 75
        fi
        if ! ZEALOT_E2E_DATABASE=postgres compose up --build --wait --remove-orphans; then
            echo "[e2e] PostgreSQL-backed server stack could not be started" >&2
            return 75
        fi
    else
        echo "[e2e] Starting SQLite-backed stack"
        ZEALOT_E2E_DATABASE=sqlite compose up --build --wait --remove-orphans
    fi

    echo "[e2e] Waiting for backend health at $SERVER_URL/health ($database)"
    if ! wait_for_http "$SERVER_URL/health" "ok"; then
        [ "$database" = "postgres" ] && return 75
        return 1
    fi

    echo "[e2e] Waiting for backend readiness at $SERVER_URL/health/ready ($database)"
    if ! wait_for_http "$SERVER_URL/health/ready" "ready"; then
        [ "$database" = "postgres" ] && return 75
        return 1
    fi

    echo "[e2e] Waiting for web root at $WEB_URL/ ($database)"
    if ! wait_for_http "$WEB_URL/"; then
        [ "$database" = "postgres" ] && return 75
        return 1
    fi

    echo "[e2e] Waiting for proxied API health at $WEB_URL/api/health ($database)"
    if ! wait_for_http "$WEB_URL/api/health" "ok"; then
        [ "$database" = "postgres" ] && return 75
        return 1
    fi

    echo "[e2e] Running pytest ($database)"
    cd "$REPO_ROOT"
    ZEALOT_E2E_DATABASE="$database" \
    ZEALOT_E2E_SERVER_URL="$SERVER_URL" \
    ZEALOT_E2E_WEB_URL="$WEB_URL" \
    ZEALOT_E2E_POSTGRES_DSN="$POSTGRES_DSN" \
    "$PYTHON_BIN" -m pytest "$SCRIPT_DIR" "$@"
}

if [ ! -x "$PYTHON_BIN" ]; then
    echo "[e2e] Creating virtualenv at $VENV_DIR"
    python3 -m venv "$VENV_DIR"
fi

if [ ! -f "$STAMP_FILE" ] || [ "$REQUIREMENTS_FILE" -nt "$STAMP_FILE" ]; then
    echo "[e2e] Installing Python requirements"
    "$PYTHON_BIN" -m pip install -r "$REQUIREMENTS_FILE"
    touch "$STAMP_FILE"
fi

trap cleanup EXIT INT TERM

case "${ZEALOT_E2E_DATABASE:-all}" in
    sqlite)
        run_backend sqlite "$@"
        ;;
    postgres)
        run_backend postgres "$@"
        ;;
    all)
        run_backend sqlite "$@"
        if run_backend postgres "$@"; then
            :
        elif [ "$?" -eq 75 ]; then
            echo "[e2e] PostgreSQL run unavailable; SQLite run remains the usable default" >&2
        else
            echo "[e2e] PostgreSQL tests failed" >&2
            exit 1
        fi
        ;;
    *)
        echo "[e2e] ZEALOT_E2E_DATABASE must be sqlite, postgres, or all" >&2
        exit 2
        ;;
esac

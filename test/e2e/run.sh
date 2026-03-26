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
                return 0
            fi
        fi
        attempts=$((attempts + 1))
        sleep 2
    done

    echo "[e2e] Timed out waiting for $url" >&2
    return 1
}

cleanup() {
    echo "[e2e] Stopping docker compose stack"
    compose down -v --remove-orphans
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

echo "[e2e] Starting docker compose stack from $COMPOSE_FILE"
compose up --build -d --remove-orphans
compose ps

echo "[e2e] Waiting for backend health at $SERVER_URL/health"
wait_for_http "$SERVER_URL/health" "ok"

echo "[e2e] Waiting for backend readiness at $SERVER_URL/health/ready"
wait_for_http "$SERVER_URL/health/ready" "ready"

echo "[e2e] Waiting for web root at $WEB_URL/"
wait_for_http "$WEB_URL/"

echo "[e2e] Waiting for proxied API health at $WEB_URL/api/health"
wait_for_http "$WEB_URL/api/health" "ok"

echo "[e2e] Running pytest"
cd "$REPO_ROOT"
ZEALOT_E2E_SERVER_URL="$SERVER_URL" \
ZEALOT_E2E_WEB_URL="$WEB_URL" \
"$PYTHON_BIN" -m pytest "$SCRIPT_DIR" "$@"

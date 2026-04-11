#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
COMPOSE_FILE="$SCRIPT_DIR/compose.yml"
WEB_URL="http://127.0.0.1:18081"

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

    echo "[playwright] Timed out waiting for $url" >&2
    return 1
}

cleanup() {
    echo "[playwright] Stopping docker compose stack"
    compose down --remove-orphans
}

trap cleanup EXIT INT TERM

# Tear down any leftover containers from a previous run so we get a fresh database.
echo "[playwright] Removing any leftover containers"
compose down --remove-orphans

echo "[playwright] Building and starting docker compose stack"
compose up --build -d

echo "[playwright] Waiting for API health at $WEB_URL/api/health"
wait_for_http "$WEB_URL/api/health" "ok"
wait_for_http "$WEB_URL/api/health/ready" "ready"

echo "[playwright] Waiting for web root at $WEB_URL/"
wait_for_http "$WEB_URL/"

echo "[playwright] Stack is ready. Running Playwright tests..."
cd "$SCRIPT_DIR"

# Install dependencies if node_modules is missing
if [ ! -d node_modules ]; then
    echo "[playwright] Installing npm dependencies"
    npm install
fi

# Install Playwright browsers if not present
if [ ! -d node_modules/.cache/ms-playwright ] && [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo "[playwright] Installing Playwright browsers"
    npx playwright install chromium
fi

PLAYWRIGHT_BASE_URL="$WEB_URL" npx playwright test "$@"

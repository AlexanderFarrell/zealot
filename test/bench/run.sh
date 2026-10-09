#!/bin/sh
# Z221 benchmark: seed a database of N items and measure edit/query latency.
#
#   ./test/bench/run.sh sqlite   [--scales 1000,10000,100000,1000000] [bench.py run options]
#   ./test/bench/run.sh postgres [--scales ...]
#
# Needs: cargo, python3, sqlite3 (sqlite runs) or docker (postgres runs).
# The server runs natively (release build). PostgreSQL runs in Docker.
# Never point this at a real Zealot: it creates and deletes its own databases
# under test/bench/.work and a throwaway container.
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
WORK_DIR=${BENCH_WORK_DIR:-"$SCRIPT_DIR/.work"}
RESULTS_DIR=${BENCH_RESULTS_DIR:-"$SCRIPT_DIR/results"}
VENV_DIR="$SCRIPT_DIR/.venv"
PY="$VENV_DIR/bin/python"
PORT=${BENCH_PORT:-47213}
URL="http://127.0.0.1:$PORT"
PG_PORT=${BENCH_PG_PORT:-47214}
PG_IMAGE=${BENCH_PG_IMAGE:-postgres:16-alpine}
PG_NAME=zealot-bench-pg
TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
SERVER_BIN=${ZEALOT_SERVER_BIN:-"$TARGET_DIR/release/zealot-server"}

DB=${1:-}
case "$DB" in
    sqlite|postgres) shift ;;
    *) echo "usage: $0 sqlite|postgres [--scales a,b,c] [bench.py run options]" >&2; exit 2 ;;
esac

SCALES="1000,10000,100000,1000000"
if [ "${1:-}" = "--scales" ]; then
    SCALES=$2
    shift 2
fi

SERVER_PID=""

log() { echo "[bench] $*" >&2; }

stop_server() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
        SERVER_PID=""
    fi
}

stop_pg() {
    docker rm -f "$PG_NAME" >/dev/null 2>&1 || true
    docker volume rm -f "$PG_NAME-data" >/dev/null 2>&1 || true
}

cleanup() {
    stop_server
    if [ "$DB" = postgres ]; then stop_pg; fi
}
trap cleanup EXIT INT TERM

if [ ! -x "$PY" ]; then
    log "creating virtualenv at $VENV_DIR"
    python3 -m venv "$VENV_DIR"
    "$PY" -m pip install -q -r "$SCRIPT_DIR/requirements.txt"
fi

if [ ! -x "$SERVER_BIN" ]; then
    log "building release server"
    (cd "$REPO_ROOT" && cargo build --release -p zealot-server)
fi

start_server() {
    mkdir -p "$WORK_DIR/public"
    if [ "$DB" = sqlite ]; then
        DATABASE=sqlite DB_FILENAME="$WORK_DIR/zealot.db" MEDIA_SOURCE=FILESYSTEM MEDIA_PATH="$WORK_DIR/public" \
            PORT="$PORT" RUST_LOG=warn BACKUP_ENABLED=false "$SERVER_BIN" >>"$WORK_DIR/server.log" 2>&1 &
    else
        DATABASE=postgres DB_HOST=127.0.0.1 PGPORT="$PG_PORT" DB_USERNAME=zealot DB_PASSWORD=zealot DB_DATABASE=zealot \
            MEDIA_SOURCE=FILESYSTEM MEDIA_PATH="$WORK_DIR/public" PORT="$PORT" RUST_LOG=warn BACKUP_ENABLED=false \
            "$SERVER_BIN" >>"$WORK_DIR/server.log" 2>&1 &
    fi
    SERVER_PID=$!
    attempts=0
    until curl -fsS "$URL/health" >/dev/null 2>&1; do
        attempts=$((attempts + 1))
        if [ "$attempts" -gt 120 ]; then log "server did not become healthy"; tail -20 "$WORK_DIR/server.log" >&2; exit 1; fi
        sleep 1
    done
}

start_pg() {
    stop_pg
    docker run -d --name "$PG_NAME" -p "$PG_PORT:5432" -v "$PG_NAME-data:/var/lib/postgresql/data" \
        -e POSTGRES_USER=zealot -e POSTGRES_PASSWORD=zealot -e POSTGRES_DB=zealot "$PG_IMAGE" \
        -c shared_buffers=1GB -c max_wal_size=8GB -c synchronous_commit=on >/dev/null
    attempts=0
    until docker exec "$PG_NAME" pg_isready -U zealot -d zealot >/dev/null 2>&1; do
        attempts=$((attempts + 1))
        if [ "$attempts" -gt 60 ]; then log "postgres did not become ready"; exit 1; fi
        sleep 1
    done
    sleep 2
}

account_id() {
    if [ "$DB" = sqlite ]; then
        sqlite3 "$WORK_DIR/zealot.db" "select account_id from account where username='bench'"
    else
        docker exec "$PG_NAME" psql -U zealot -d zealot -Atc "select account_id from account where username='bench'"
    fi
}

seed() {
    n=$1
    acct=$2
    sed -e "s/@N@/$n/g" -e "s/@ACCOUNT@/$acct/g" "$SCRIPT_DIR/seed.$DB.sql" >"$WORK_DIR/seed.sql"
    if [ "$DB" = sqlite ]; then
        sqlite3 "$WORK_DIR/zealot.db" <"$WORK_DIR/seed.sql"
    else
        docker exec -i "$PG_NAME" psql -U zealot -d zealot -v ON_ERROR_STOP=1 -q <"$WORK_DIR/seed.sql"
    fi
}

db_size() {
    if [ "$DB" = sqlite ]; then
        du -sh "$WORK_DIR/zealot.db"* 2>/dev/null | awk '{print $1}' | tr '\n' ' '
    else
        docker exec "$PG_NAME" psql -U zealot -d zealot -Atc "select pg_size_pretty(pg_database_size('zealot'))"
    fi
}

mkdir -p "$RESULTS_DIR"
for N in $(echo "$SCALES" | tr ',' ' '); do
    log "=== $DB, $N items ==="
    rm -rf "$WORK_DIR"
    mkdir -p "$WORK_DIR"
    if [ "$DB" = postgres ]; then start_pg; fi
    start_server
    "$PY" "$SCRIPT_DIR/bench.py" prepare --url "$URL"
    ACCT=$(account_id)
    stop_server
    t0=$(date +%s)
    seed "$N" "$ACCT"
    SEED_S=$(($(date +%s) - t0))
    log "seeded $N items in ${SEED_S}s; database size: $(db_size)"
    start_server
    "$PY" "$SCRIPT_DIR/bench.py" check --url "$URL" --scale "$N"
    "$PY" "$SCRIPT_DIR/bench.py" run --url "$URL" --scale "$N" --db "$DB" --out "$RESULTS_DIR/$DB-$N.json" "$@" \
        | tee "$RESULTS_DIR/$DB-$N.md"
    tail -200 "$WORK_DIR/server.log" >"$RESULTS_DIR/$DB-$N.server.log" 2>/dev/null || true
    log "server RSS after run: $(ps -o rss= -p "$SERVER_PID" 2>/dev/null | awk '{printf "%.0f MB", $1/1024}')"
    printf '\nseed_seconds: %s\ndatabase_size: %s\n' "$SEED_S" "$(db_size)" >>"$RESULTS_DIR/$DB-$N.md"
    stop_server
    if [ "$DB" = postgres ]; then stop_pg; fi
done

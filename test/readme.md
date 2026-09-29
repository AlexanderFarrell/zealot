# E2E Smoke Tests

## Fastest path

Run the suite from the repo root with:

```sh
./test/e2e/run.sh
```

That script will:

1. Create `test/e2e/.venv` if it does not exist.
2. Install `test/e2e/requirements.txt` into that virtualenv when needed.
3. Bring up the dedicated e2e Docker stack from `test/e2e/compose.yml`.
4. Wait for the backend and web health endpoints to come up.
5. Run `pytest test/e2e` through the virtualenv interpreter with normal pytest output.
6. Tear the e2e Docker stack back down when the run finishes.

## Optional npm shortcut

From the repo root:

```sh
npm run test:e2e
```

Pass extra pytest arguments through either entrypoint:

```sh
./test/e2e/run.sh -k health -q
npm run test:e2e -- -k health
```

## Manual commands

If you want to run the commands yourself instead of the helper script:

```sh
python3 -m venv test/e2e/.venv
test/e2e/.venv/bin/pip install -r test/e2e/requirements.txt
docker compose -f test/e2e/compose.yml up --build -d --remove-orphans
test/e2e/.venv/bin/pytest test/e2e -q
docker compose -f test/e2e/compose.yml down -v --remove-orphans
```

If you want `pytest` available directly in your shell for the current session:

```sh
source test/e2e/.venv/bin/activate
docker compose -f test/e2e/compose.yml up --build -d --remove-orphans
pytest test/e2e -q
docker compose -f test/e2e/compose.yml down -v --remove-orphans
```

## Requirements

- Docker Desktop or another working Docker Engine with `docker compose`
- Python 3
- Network access the first time the virtualenv installs `pytest` and `requests`
- `curl` available in your shell, because the helper script uses it for health checks

## Compose split

- General local stack: `scripts/zealot-compose.yml` plus `scripts/build-zealot-compose.yml`
- E2E-only stack: `test/e2e/compose.yml`

The e2e stack uses separate host ports so it does not collide with the general local stack:

- Web: `http://127.0.0.1:18080`
- Server: `http://127.0.0.1:18456`

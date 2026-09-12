from __future__ import annotations

import os
import sqlite3
import time
import uuid

import pytest
import requests

from test_rules_engine import create_rule, csrf_headers, wait_for_rule_run


DB_PATH = os.path.join(os.path.dirname(__file__), "zealot_data", "zealot.db")
PASSWORD = "correct horse battery staple"


@pytest.fixture(scope="module")
def scope_db():
    database = os.environ.get("ZEALOT_E2E_DATABASE")
    if database == "sqlite":
        if not os.path.exists(DB_PATH):
            pytest.skip("SQLite database not found — is the stack running?")
        conn = sqlite3.connect(DB_PATH)
        conn.row_factory = sqlite3.Row
        yield conn, False
        conn.close()
        return

    if database != "postgres":
        pytest.skip("scope authorization tests require an explicit backend")

    psycopg = pytest.importorskip("psycopg")
    from psycopg.rows import dict_row

    dsn = os.environ.get("ZEALOT_E2E_POSTGRES_DSN")
    if not dsn:
        pytest.skip("ZEALOT_E2E_POSTGRES_DSN is not configured")
    try:
        conn = psycopg.connect(dsn, row_factory=dict_row)
    except psycopg.Error as exc:
        pytest.skip(f"PostgreSQL E2E service is unavailable: {exc}")
    yield conn, True
    conn.close()


def _placeholder(postgres: bool) -> str:
    return "%s" if postgres else "?"


def _uuid_value(value: str, postgres: bool):
    return uuid.UUID(value) if postgres else value


def _register(stack_urls: dict[str, str], prefix: str) -> tuple[requests.Session, int]:
    suffix = uuid.uuid4().hex[:12]
    session = requests.Session()
    response = session.post(
        f"{stack_urls['server_url']}/auth/register",
        json={
            "username": f"{prefix}_{suffix}",
            "password": PASSWORD,
            "email": f"{prefix}_{suffix}@example.com",
            "given_name": prefix.title(),
            "surname": "Scope",
        },
        timeout=5,
    )
    assert response.status_code == 200, response.text
    return session, int(response.json()["account_id"])


def _principal_scope(conn, account_id: int, postgres: bool) -> tuple[str, str]:
    mark = _placeholder(postgres)
    row = conn.execute(
        f"""
        select principal_id, default_scope_id
        from server_principal
        where account_id = {mark}
        """,
        (account_id,),
    ).fetchone()
    assert row is not None
    return str(row["principal_id"]), str(row["default_scope_id"])


def _set_membership(
    conn,
    scope_id: str,
    principal_id: str,
    role: str,
    status: str,
    postgres: bool,
) -> None:
    value = _uuid_value
    if postgres:
        conn.execute(
            """
            insert into scope_member (scope_id, principal_id, role, status)
            values (%s, %s, %s, %s)
            on conflict (scope_id, principal_id) do update
            set role = excluded.role, status = excluded.status,
                updated_at = now()
            """,
            (value(scope_id, True), value(principal_id, True), role, status),
        )
    else:
        conn.execute(
            """
            insert into scope_member (scope_id, principal_id, role, status)
            values (?, ?, ?, ?)
            on conflict (scope_id, principal_id) do update
            set role = excluded.role, status = excluded.status,
                updated_at = strftime('%s', 'now')
            """,
            (scope_id, principal_id, role, status),
        )
    conn.commit()


def _create_service_key(
    session: requests.Session,
    stack_urls: dict[str, str],
    scope_id: str,
) -> tuple[str, str]:
    response = session.post(
        f"{stack_urls['server_url']}/account/scopes/{scope_id}/service-keys",
        json={
            "display_name": "E2E scope service",
            "label": "E2E scope service key",
            "role": "viewer",
        },
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 200, response.text
    payload = response.json()
    return str(payload["principal_id"]), payload["key"]


def _create_item(
    session: requests.Session,
    stack_urls: dict[str, str],
    title: str,
    *,
    scope_id: str | None = None,
) -> requests.Response:
    params = {"scope_id": scope_id} if scope_id else None
    return session.post(
        f"{stack_urls['server_url']}/item",
        params=params,
        json={"title": title, "content": f"{title} sentinel"},
        headers=csrf_headers(session),
        timeout=5,
    )


def test_two_scope_roles_service_and_revocation(
    stack_urls: dict[str, str],
    scope_db,
) -> None:
    conn, postgres = scope_db
    base = stack_urls["server_url"]

    alice, alice_account_id = _register(stack_urls, "scope_alice")
    bob, bob_account_id = _register(stack_urls, "scope_bob")
    alice_principal, alice_scope = _principal_scope(conn, alice_account_id, postgres)
    bob_principal, bob_scope = _principal_scope(conn, bob_account_id, postgres)

    alice_item = _create_item(alice, stack_urls, "scope-alice-sentinel")
    assert alice_item.status_code == 200, alice_item.text
    bob_item = _create_item(bob, stack_urls, "scope-bob-sentinel")
    assert bob_item.status_code == 200, bob_item.text
    alice_item_id = int(alice_item.json()["item_id"])
    bob_item_id = int(bob_item.json()["item_id"])

    # Bob can read Alice's scope as a viewer, but his default scope remains
    # his own personal scope.
    _set_membership(conn, alice_scope, bob_principal, "viewer", "active", postgres)
    own = bob.get(f"{base}/item/search", params={"term": "scope-"}, timeout=5)
    assert own.status_code == 200
    own_titles = {item["title"] for item in own.json()}
    assert "scope-bob-sentinel" in own_titles
    assert "scope-alice-sentinel" not in own_titles

    explicit = bob.get(
        f"{base}/item/search",
        params={"term": "scope-", "scope_id": alice_scope},
        timeout=5,
    )
    assert explicit.status_code == 200, explicit.text
    assert {item["title"] for item in explicit.json()} == {"scope-alice-sentinel"}

    denied_write = _create_item(
        bob,
        stack_urls,
        "scope-viewer-denied",
        scope_id=alice_scope,
    )
    assert denied_write.status_code == 403, denied_write.text

    # Promotion to editor enables the item mutation without changing the
    # account's default scope selection.
    _set_membership(conn, alice_scope, bob_principal, "editor", "active", postgres)
    editor_write = _create_item(
        bob,
        stack_urls,
        "scope-editor-created",
        scope_id=alice_scope,
    )
    assert editor_write.status_code == 200, editor_write.text

    # Cross-scope IDs never disclose the other scope's sentinel item.
    hidden = alice.get(f"{base}/item/id/{bob_item_id}", timeout=5)
    assert hidden.status_code == 404, hidden.text
    denied_scope = alice.get(
        f"{base}/item/id/{alice_item_id}",
        params={"scope_id": bob_scope},
        timeout=5,
    )
    assert denied_scope.status_code == 403, denied_scope.text

    all_scopes = bob.get(
        f"{base}/item/search",
        params={"term": "scope-", "all_scopes": "true"},
        timeout=5,
    )
    assert all_scopes.status_code == 200, all_scopes.text
    all_titles = {item["title"] for item in all_scopes.json()}
    assert {"scope-alice-sentinel", "scope-bob-sentinel"} <= all_titles

    # The helper above uses the default scope; issue the explicit all-scope
    # request separately because all_scopes is a query-only read/mutation guard.
    all_scope_write = bob.post(
        f"{base}/item",
        params={"all_scopes": "true"},
        json={"title": "scope-all-write-denied", "content": "sentinel"},
        headers=csrf_headers(bob),
        timeout=5,
    )
    assert all_scope_write.status_code == 403, all_scope_write.text

    service_principal, service_key = _create_service_key(alice, stack_urls, alice_scope)
    service_headers = {"x-api-key": service_key}
    service_default = requests.get(f"{base}/item/search", params={"term": "scope-"}, headers=service_headers, timeout=5)
    assert service_default.status_code == 401, service_default.text
    service_read = requests.get(
        f"{base}/item/search",
        params={"term": "scope-", "scope_id": alice_scope},
        headers=service_headers,
        timeout=5,
    )
    assert service_read.status_code == 200, service_read.text
    assert {item["title"] for item in service_read.json()} >= {"scope-alice-sentinel"}
    service_write = requests.post(
        f"{base}/item",
        params={"scope_id": alice_scope},
        json={"title": "scope-service-write-denied", "content": "sentinel"},
        headers=service_headers,
        timeout=5,
    )
    assert service_write.status_code == 403, service_write.text

    # Revocation is observed on the next request; no process restart or cache
    # invalidation is needed.
    _set_membership(conn, alice_scope, service_principal, "viewer", "revoked", postgres)
    revoked = requests.get(
        f"{base}/item/search",
        params={"term": "scope-", "scope_id": alice_scope},
        headers=service_headers,
        timeout=5,
    )
    assert revoked.status_code == 403, revoked.text


def test_rules_are_bound_to_the_event_scope(
    stack_urls: dict[str, str],
    scope_db,
) -> None:
    conn, postgres = scope_db
    base = stack_urls["server_url"]

    alice, alice_account_id = _register(stack_urls, "rule_scope_alice")
    bob, bob_account_id = _register(stack_urls, "rule_scope_bob")
    alice_principal, alice_scope = _principal_scope(conn, alice_account_id, postgres)
    bob_principal, bob_scope = _principal_scope(conn, bob_account_id, postgres)

    rule = create_rule(
        alice,
        stack_urls,
        name=f"scope-rule-{uuid.uuid4().hex[:12]}",
        trigger={"kind": "on_item_create"},
        script='zealot.notify("alice-scope-rule-fired")',
    )
    rule_id = int(rule["rule_id"])
    assert rule["scope_id"] == alice_scope

    # Bob's item emits an event for Bob's scope only.  The rule must remain
    # untouched even though both scopes are active on the same server.
    bob_item = _create_item(bob, stack_urls, "rule-scope-bob-event")
    assert bob_item.status_code == 200, bob_item.text
    time.sleep(0.4)
    after_bob = alice.get(f"{base}/rule/{rule_id}", timeout=5)
    assert after_bob.status_code == 200, after_bob.text
    assert after_bob.json()["last_run_at"] is None

    alice_item = _create_item(alice, stack_urls, "rule-scope-alice-event")
    assert alice_item.status_code == 200, alice_item.text
    stored = wait_for_rule_run(alice, stack_urls, rule_id)
    assert stored["last_output"] == "alice-scope-rule-fired"

    # A member can read the selected scope, but a viewer cannot execute its
    # rule.  The other scope remains non-addressable by the rule ID.
    _set_membership(conn, alice_scope, bob_principal, "viewer", "active", postgres)
    selected = bob.get(
        f"{base}/rule/{rule_id}",
        params={"scope_id": alice_scope},
        timeout=5,
    )
    assert selected.status_code == 200, selected.text
    denied_run = bob.post(
        f"{base}/rule/{rule_id}/run",
        params={"scope_id": alice_scope},
        json={},
        headers=csrf_headers(bob),
        timeout=5,
    )
    assert denied_run.status_code == 403, denied_run.text
    hidden = bob.get(f"{base}/rule/{rule_id}", params={"scope_id": bob_scope}, timeout=5)
    assert hidden.status_code == 404, hidden.text

    service_principal, service_key = _create_service_key(alice, stack_urls, alice_scope)
    service_headers = {"x-api-key": service_key}
    service_default = requests.get(f"{base}/rule", headers=service_headers, timeout=5)
    assert service_default.status_code == 401, service_default.text
    service_selected = requests.get(
        f"{base}/rule",
        params={"scope_id": alice_scope},
        headers=service_headers,
        timeout=5,
    )
    assert service_selected.status_code == 200, service_selected.text
    assert any(int(item["rule_id"]) == rule_id for item in service_selected.json())
    service_run = requests.post(
        f"{base}/rule/{rule_id}/run",
        params={"scope_id": alice_scope},
        json={},
        headers=service_headers,
        timeout=5,
    )
    assert service_run.status_code == 403, service_run.text
    assert service_principal

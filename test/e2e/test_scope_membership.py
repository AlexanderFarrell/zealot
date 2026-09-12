from __future__ import annotations

import os
import sqlite3
import uuid

import pytest
import requests

from test_rules_engine import csrf_headers
from test_scope_authorization import _principal_scope, _register


DB_PATH = os.path.join(os.path.dirname(__file__), "zealot_data", "zealot.db")


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
        pytest.skip("scope membership tests require an explicit backend")
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


def _db_value(value: str, postgres: bool):
    return uuid.UUID(value) if postgres else value


def test_invitation_acceptance_is_single_use_and_audited(
    stack_urls: dict[str, str], scope_db
) -> None:
    conn, postgres = scope_db
    base = stack_urls["server_url"]
    owner, owner_account_id = _register(stack_urls, "member_owner")
    recipient, recipient_account_id = _register(stack_urls, "member_recipient")
    owner_principal, scope_id = _principal_scope(conn, owner_account_id, postgres)
    recipient_principal, _ = _principal_scope(conn, recipient_account_id, postgres)

    created = owner.post(
        f"{base}/account/scopes/{scope_id}/invitations",
        json={
            "role": "editor",
            "recipient_principal_id": recipient_principal,
            "ttl_seconds": 300,
        },
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert created.status_code == 200, created.text
    payload = created.json()
    token = payload["token"]
    invitation = payload["invitation"]
    assert token.startswith("z1.")
    assert invitation["status"] == "pending"

    listed = owner.get(f"{base}/account/scopes/{scope_id}/invitations", timeout=5)
    assert listed.status_code == 200, listed.text
    assert listed.json()[0]["invitation_id"] == invitation["invitation_id"]
    assert all("token" not in item for item in listed.json())

    accepted = recipient.post(
        f"{base}/account/invitations/accept",
        json={"token": token},
        headers=csrf_headers(recipient),
        timeout=5,
    )
    assert accepted.status_code == 200, accepted.text
    assert accepted.json()["status"] == "active"

    replay = recipient.post(
        f"{base}/account/invitations/accept",
        json={"token": token},
        headers=csrf_headers(recipient),
        timeout=5,
    )
    assert replay.status_code == 409, replay.text

    members = owner.get(f"{base}/account/scopes/{scope_id}/members", timeout=5)
    assert members.status_code == 200, members.text
    matching = [item for item in members.json() if item["principal_id"] == recipient_principal]
    assert len(matching) == 1
    assert matching[0]["role"] == "editor"
    assert matching[0]["status"] == "active"

    denied_create = recipient.post(
        f"{base}/account/scopes/{scope_id}/invitations",
        json={"role": "viewer", "ttl_seconds": 300},
        headers=csrf_headers(recipient),
        timeout=5,
    )
    assert denied_create.status_code == 404, denied_create.text

    changed = owner.patch(
        f"{base}/account/scopes/{scope_id}/members/{recipient_principal}",
        json={"role": "viewer"},
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert changed.status_code == 200, changed.text
    assert changed.json()["role"] == "viewer"

    revoked = owner.delete(
        f"{base}/account/scopes/{scope_id}/members/{recipient_principal}",
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert revoked.status_code == 204, revoked.text

    final_owner = owner.delete(
        f"{base}/account/scopes/{scope_id}/members/{owner_principal}",
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert final_owner.status_code == 409, final_owner.text

    mark = "%s" if postgres else "?"
    events = conn.execute(
        f"select event_type, outcome, reason_code, correlation_id from scope_lifecycle_event where scope_id = {mark}",
        (_db_value(scope_id, postgres),),
    ).fetchall()
    event_types = {str(row["event_type"]) for row in events}
    assert {
        "invitation_created",
        "invitation_accepted",
        "membership_role_changed",
        "membership_revoked",
    } <= event_types
    assert all(token not in " ".join(str(value) for value in row) for row in events)
    assert all(row["correlation_id"] for row in events)


def test_pending_admission_expiry_and_cancellation_do_not_activate_membership(
    stack_urls: dict[str, str], scope_db
) -> None:
    conn, postgres = scope_db
    base = stack_urls["server_url"]
    owner, owner_account_id = _register(stack_urls, "admission_owner")
    owner_principal, scope_id = _principal_scope(conn, owner_account_id, postgres)

    pending_response = owner.post(
        f"{base}/account/scopes/{scope_id}/invitations",
        json={"role": "viewer", "ttl_seconds": 300},
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert pending_response.status_code == 200, pending_response.text
    pending_payload = pending_response.json()
    pending_token = pending_payload["token"]
    pending_id = pending_payload["invitation"]["invitation_id"]

    anonymous = requests.post(
        f"{base}/account/invitations/accept",
        json={"token": pending_token},
        timeout=5,
    )
    assert anonymous.status_code == 202, anonymous.text
    assert anonymous.json()["status"] == "pending_admission"

    cancelled = owner.post(
        f"{base}/account/scopes/{scope_id}/invitations/{pending_id}/cancel",
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert cancelled.status_code == 204, cancelled.text
    cancelled_replay = requests.post(
        f"{base}/account/invitations/accept",
        json={"token": pending_token},
        timeout=5,
    )
    assert cancelled_replay.status_code == 409, cancelled_replay.text

    expiring_response = owner.post(
        f"{base}/account/scopes/{scope_id}/invitations",
        json={"role": "viewer", "ttl_seconds": 300},
        headers=csrf_headers(owner),
        timeout=5,
    )
    assert expiring_response.status_code == 200, expiring_response.text
    expiring_payload = expiring_response.json()
    expiring_token = expiring_payload["token"]
    expiring_id = expiring_payload["invitation"]["invitation_id"]
    mark = "%s" if postgres else "?"
    if postgres:
        conn.execute(
            f"update scope_invitation set expires_at = now() - interval '1 second' where invitation_id = {mark}",
            (_db_value(expiring_id, postgres),),
        )
    else:
        conn.execute(
            f"update scope_invitation set expires_at = 1 where invitation_id = {mark}",
            (_db_value(expiring_id, postgres),),
        )
    conn.commit()
    expired = requests.post(
        f"{base}/account/invitations/accept",
        json={"token": expiring_token},
        timeout=5,
    )
    assert expired.status_code == 409, expired.text

    owner_members = owner.get(f"{base}/account/scopes/{scope_id}/members", timeout=5)
    assert owner_members.status_code == 200, owner_members.text
    assert all(item["principal_id"] == owner_principal for item in owner_members.json())

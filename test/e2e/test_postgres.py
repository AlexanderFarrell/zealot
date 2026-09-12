from __future__ import annotations

import os
import uuid

import pytest
import requests

psycopg = pytest.importorskip("psycopg")
from psycopg.rows import dict_row


EXPECTED_MIGRATIONS = [
    "initial schema",
    "account split full name",
    "backfill parent links",
    "rules",
    "relationship string",
    "api key",
    "template item type",
    "add status values",
    "cascade delete comments",
    "api keys multi",
    "heading and link index",
    "item views",
    "time blocks",
    "statistics",
    "reorder status values",
    "core model foundation",
    "default scope invariants",
    "scope authorization",
    "rule scope",
    "rule scope invariants",
    "scope invitations membership",
]


@pytest.fixture(scope="module")
def db():
    if os.environ.get("ZEALOT_E2E_DATABASE") != "postgres":
        pytest.skip("PostgreSQL tests require ZEALOT_E2E_DATABASE=postgres")

    dsn = os.environ.get("ZEALOT_E2E_POSTGRES_DSN")
    if not dsn:
        pytest.skip("ZEALOT_E2E_POSTGRES_DSN is not configured")

    try:
        conn = psycopg.connect(dsn, row_factory=dict_row)
    except psycopg.Error as exc:
        pytest.skip(f"PostgreSQL E2E service is unavailable: {exc}")
    yield conn
    conn.close()


def columns(db, table: str) -> set[str]:
    rows = db.execute(
        """
        select column_name
        from information_schema.columns
        where table_schema = 'public' and table_name = %s
        """,
        (table,),
    ).fetchall()
    return {row["column_name"] for row in rows}


def test_postgres_migrations_are_complete(db) -> None:
    rows = db.execute(
        "select description from _sqlx_migrations order by version"
    ).fetchall()
    assert [row["description"] for row in rows] == EXPECTED_MIGRATIONS


def test_postgres_scope_foundation_schema_is_present(db) -> None:
    rows = db.execute(
        """
        select table_name
        from information_schema.tables
        where table_schema = 'public'
        """
    ).fetchall()
    tables = {row["table_name"] for row in rows}
    assert {"server", "server_principal", "scope", "scope_member"} <= tables
    assert {
        "principal_id",
        "server_id",
        "kind",
        "account_id",
        "default_scope_id",
    } <= columns(db, "server_principal")
    assert "scope_id" in columns(db, "item")
    assert "scope_id" in columns(db, "rule")


def test_postgres_scope_membership_schema_is_present(db) -> None:
    rows = db.execute(
        """
        select table_name
        from information_schema.tables
        where table_schema = 'public'
        """
    ).fetchall()
    assert {"scope_invitation", "scope_lifecycle_event"} <= {
        row["table_name"] for row in rows
    }
    assert {"enrolment_policy", "invitation_signing_key"} <= columns(db, "server")
    assert {
        "scope_id",
        "issuer_principal_id",
        "recipient_principal_id",
        "token_hash",
        "token_signature",
        "status",
        "expires_at",
        "correlation_id",
    } <= columns(db, "scope_invitation")


def test_postgres_scope_foundation_bootstrap_is_consistent(db) -> None:
    server_rows = db.execute("select server_id, display_name from server").fetchall()
    assert len(server_rows) == 1
    assert server_rows[0]["display_name"] == "Zealot server"

    human_principals = db.execute(
        """
        select p.principal_id, p.account_id, p.default_scope_id,
               s.scope_id, sm.role, sm.status
        from server_principal p
        join scope s on s.scope_id = p.default_scope_id
        join scope_member sm on sm.scope_id = s.scope_id
                            and sm.principal_id = p.principal_id
        where p.kind = 'human'
        """
    ).fetchall()
    for principal in human_principals:
        assert principal["account_id"] is not None
        assert principal["default_scope_id"] == principal["scope_id"]
        assert principal["role"] == "owner"
        assert principal["status"] == "active"

    assert db.execute(
        "select count(*) as count from item where scope_id is null"
    ).fetchone()["count"] == 0
    assert db.execute(
        "select count(*) as count from rule where scope_id is null"
    ).fetchone()["count"] == 0


def test_postgres_new_account_gets_personal_default_scope(db, stack_urls) -> None:
    suffix = uuid.uuid4().hex[:12]
    response = requests.post(
        f"{stack_urls['server_url']}/auth/register",
        json={
            "username": f"pg_scope_{suffix}",
            "password": "correct horse battery staple",
            "email": f"pg_scope_{suffix}@example.com",
            "given_name": "Postgres",
            "surname": "Scope",
        },
        timeout=5,
    )
    assert response.status_code == 200
    account_id = response.json()["account_id"]

    row = db.execute(
        """
        select p.kind, p.account_id, p.default_scope_id,
               s.title, sm.role, sm.status
        from server_principal p
        join scope s on s.scope_id = p.default_scope_id
        join scope_member sm on sm.scope_id = s.scope_id
                            and sm.principal_id = p.principal_id
        where p.account_id = %s
        """,
        (account_id,),
    ).fetchone()
    assert row is not None
    assert row["kind"] == "human"
    assert row["account_id"] == account_id
    assert row["title"].startswith("Personal — pg_scope_")
    assert row["role"] == "owner"
    assert row["status"] == "active"

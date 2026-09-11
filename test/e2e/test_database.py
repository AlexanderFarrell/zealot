from __future__ import annotations

import os
import sqlite3

import pytest
import requests


# The compose bind-mount puts the SQLite file at zealot_data/zealot.db
# relative to this test file.
DB_PATH = os.path.join(os.path.dirname(__file__), "zealot_data", "zealot.db")

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
]

EXPECTED_SYSTEM_ATTRIBUTE_KINDS = [
    "Date",
    "Status",
    "Week",
    "Priority",
    "Month",
    "Year",
    "Root",
    "Parent",
    "Time of Day",
    "Phone",
    "Email",
    "Schedule",
    "End Date",
    "Value Kind",
    "Unit",
    "Daily Aggregation",
]

EXPECTED_ITEM_TYPES = ["Plan", "Repeat", "Template", "Statistic"]


@pytest.fixture(scope="module")
def db() -> sqlite3.Connection:
    if not os.path.exists(DB_PATH):
        pytest.skip(f"SQLite database not found at {DB_PATH} — is the stack running?")
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    yield conn
    conn.close()


def test_all_migrations_applied(db: sqlite3.Connection) -> None:
    rows = db.execute("select description from _sqlx_migrations order by version").fetchall()
    applied = [row["description"] for row in rows]
    assert applied == EXPECTED_MIGRATIONS


def test_system_attribute_kinds_seeded(db: sqlite3.Connection) -> None:
    rows = db.execute("select key from attribute_kind where is_system = 1").fetchall()
    seeded = {row["key"] for row in rows}
    assert seeded == set(EXPECTED_SYSTEM_ATTRIBUTE_KINDS)


def test_item_types_seeded(db: sqlite3.Connection) -> None:
    rows = db.execute("select name from item_type where account_id is null").fetchall()
    seeded = {row["name"] for row in rows}
    assert seeded == set(EXPECTED_ITEM_TYPES)


def test_item_type_attribute_kind_links_seeded(db: sqlite3.Connection) -> None:
    # Plan should be linked to Status and Priority
    rows = db.execute("""
        select ak.key
        from item_type_attribute_kind_link link
        join attribute_kind ak on ak.kind_id = link.attribute_kind_id
        join item_type it on it.type_id = link.item_type_id
        where it.name = 'Plan'
    """).fetchall()
    plan_kinds = {row["key"] for row in rows}
    assert "Status" in plan_kinds
    assert "Priority" in plan_kinds

    # Repeat should be linked to Schedule
    rows = db.execute("""
        select ak.key
        from item_type_attribute_kind_link link
        join attribute_kind ak on ak.kind_id = link.attribute_kind_id
        join item_type it on it.type_id = link.item_type_id
        where it.name = 'Repeat'
    """).fetchall()
    repeat_kinds = {row["key"] for row in rows}
    assert "Schedule" in repeat_kinds

    rows = db.execute("""
        select ak.key
        from item_type_attribute_kind_link link
        join attribute_kind ak on ak.kind_id = link.attribute_kind_id
        join item_type it on it.type_id = link.item_type_id
        where it.name = 'Statistic'
    """).fetchall()
    statistic_kinds = {row["key"] for row in rows}
    assert statistic_kinds == {"Value Kind", "Unit", "Daily Aggregation"}


def test_statistic_entry_schema(db: sqlite3.Connection) -> None:
    cols = {row[1] for row in db.execute("pragma table_info(statistic_entry)").fetchall()}
    assert cols == {
        "statistic_entry_id",
        "item_id",
        "account_id",
        "value",
        "occurred_at",
        "related_item_id",
        "comment",
        "created_at",
        "updated_at",
    }
    table_sql = db.execute(
        "select sql from sqlite_master where type = 'table' and name = 'statistic_entry'"
    ).fetchone()[0]
    assert "abs(value) <= 1.7976931348623157e308" in table_sql
    indexes = {
        row[1] for row in db.execute("pragma index_list(statistic_entry)").fetchall()
    }
    assert "idx_statistic_entry_item_time" in indexes


def test_account_table_has_given_name_and_surname(db: sqlite3.Connection) -> None:
    cols = {row[1] for row in db.execute("pragma table_info(account)").fetchall()}
    assert "given_name" in cols
    assert "surname" in cols
    assert "full_name" not in cols


def test_scope_foundation_schema_is_present(db: sqlite3.Connection) -> None:
    tables = {
        row["name"]
        for row in db.execute(
            "select name from sqlite_master where type = 'table'"
        ).fetchall()
    }
    assert {"server", "server_principal", "scope", "scope_member"} <= tables

    principal_columns = {
        row["name"]
        for row in db.execute("pragma table_info(server_principal)").fetchall()
    }
    assert {
        "principal_id",
        "server_id",
        "kind",
        "account_id",
        "default_scope_id",
    } <= principal_columns

    item_columns = {row["name"] for row in db.execute("pragma table_info(item)").fetchall()}
    assert "scope_id" in item_columns


def test_scope_foundation_bootstrap_is_consistent(db: sqlite3.Connection) -> None:
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

    # The migration must not leave any item unscoped, including the empty
    # account state produced before the first login/register request.
    unscoped_items = db.execute(
        "select count(*) from item where scope_id is null"
    ).fetchone()[0]
    assert unscoped_items == 0


def test_new_account_gets_personal_default_scope(
    db: sqlite3.Connection,
    stack_urls: dict[str, str],
) -> None:
    # Register through the public compatibility path, then verify the same
    # persisted principal/scope/member shape that legacy accounts receive.
    import uuid

    suffix = uuid.uuid4().hex[:12]
    response = requests.post(
        f"{stack_urls['server_url']}/auth/register",
        json={
            "username": f"scope_{suffix}",
            "password": "correct horse battery staple",
            "email": f"scope_{suffix}@example.com",
            "given_name": "Scope",
            "surname": "Fixture",
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
        where p.account_id = ?
        """,
        (account_id,),
    ).fetchone()
    assert row is not None
    assert row["kind"] == "human"
    assert row["account_id"] == account_id
    assert row["title"].startswith("Personal — scope_")
    assert row["role"] == "owner"
    assert row["status"] == "active"

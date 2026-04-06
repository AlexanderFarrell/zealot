from __future__ import annotations

import os
import sqlite3

import pytest


# The compose bind-mount puts the SQLite file at zealot_data/zealot.db
# relative to this test file.
DB_PATH = os.path.join(os.path.dirname(__file__), "zealot_data", "zealot.db")

EXPECTED_MIGRATIONS = [
    "initial schema",
    "account split full name",
    "backfill parent links",
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
]

EXPECTED_ITEM_TYPES = ["Plan", "Repeat"]


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


def test_account_table_has_given_name_and_surname(db: sqlite3.Connection) -> None:
    cols = {row[1] for row in db.execute("pragma table_info(account)").fetchall()}
    assert "given_name" in cols
    assert "surname" in cols
    assert "full_name" not in cols

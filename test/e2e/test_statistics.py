from __future__ import annotations

from test_rules_engine import create_item, create_rule, csrf_headers, register_session, run_rule


def test_statistic_crud_aggregation_and_lua(stack_urls: dict[str, str]) -> None:
    session = register_session(stack_urls)
    base = stack_urls["server_url"]
    related = create_item(session, stack_urls, title="Statistic context")
    statistic = create_item(
        session,
        stack_urls,
        title="Body Weight",
        attributes={
            "Value Kind": "Number",
            "Unit": "kg",
            "Daily Aggregation": "Average",
        },
        types=["Statistic"],
    )
    item_id = int(statistic["item_id"])

    first = session.post(
        f"{base}/statistic/{item_id}/entries",
        json={
            "value": 80.0,
            "occurred_at": "2026-08-12T23:30:00-02:00",
            "related_item_id": related["item_id"],
            "comment": "First",
        },
        headers=csrf_headers(session),
        timeout=5,
    )
    assert first.status_code == 201, first.text
    first_entry = first.json()
    assert first_entry["occurred_at"].startswith("2026-08-13T01:30:00")

    second = session.post(
        f"{base}/statistic/{item_id}/entries",
        json={"value": 82.0, "occurred_at": "2026-08-13T10:00:00Z"},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert second.status_code == 201, second.text

    listed = session.get(
        f"{base}/statistic/{item_id}/entries",
        params={
            "start": "2026-08-13T00:00:00Z",
            "end": "2026-08-14T00:00:00Z",
            "limit": 1,
            "offset": 0,
        },
        timeout=5,
    )
    assert listed.status_code == 200, listed.text
    page = listed.json()
    assert page["count"] == 1
    assert page["next_offset"] == 1
    assert page["entries"][0]["value"] == 82.0

    daily = session.get(
        f"{base}/statistic/{item_id}/daily",
        params={"start": "2026-08-13T00:00:00Z", "end": "2026-08-14T00:00:00Z"},
        timeout=5,
    )
    assert daily.status_code == 200, daily.text
    assert daily.json() == [{"date": "2026-08-13", "value": 81.0, "count": 2}]

    summary = session.get(
        f"{base}/statistic/{item_id}/summary",
        params={"start": "2026-08-13T00:00:00Z", "end": "2026-08-14T00:00:00Z"},
        timeout=5,
    )
    assert summary.status_code == 200, summary.text
    assert summary.json()["delta"] == 2.0
    assert summary.json()["sum"] == 162.0

    patched = session.patch(
        f"{base}/statistic/entries/{first_entry['statistic_entry_id']}",
        json={"value": 81.0, "related_item_id": None, "comment": None},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert patched.status_code == 200, patched.text
    assert patched.json()["related_item_id"] is None
    assert patched.json()["comment"] is None

    invalid_owner = create_item(session, stack_urls, title="Not a Statistic")
    invalid = session.post(
        f"{base}/statistic/{invalid_owner['item_id']}/entries",
        json={"value": 1},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert invalid.status_code == 400

    rule = create_rule(
        session,
        stack_urls,
        name="Statistic Lua contract",
        trigger={"kind": "manual"},
        script=f"""
local entry = zealot.statistics.record({item_id}, 83.0, {{
    occurred_at = "2026-08-13T12:00:00Z"
}})
local summary = zealot.statistics.summary({item_id}, {{
    start = "2026-08-13T00:00:00Z",
    ["end"] = "2026-08-14T00:00:00Z"
}})
zealot.notify("count=" .. summary.count .. ",latest=" .. summary.latest.value)
""".strip(),
    )
    result = run_rule(session, stack_urls, int(rule["rule_id"]))
    assert result["success"] is True, result
    assert result["output"] == "count=3,latest=83.0"

    deleted = session.delete(
        f"{base}/statistic/entries/{first_entry['statistic_entry_id']}",
        headers=csrf_headers(session),
        timeout=5,
    )
    assert deleted.status_code == 204, deleted.text

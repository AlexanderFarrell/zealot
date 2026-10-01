from __future__ import annotations

import time
import uuid
from urllib.parse import quote

import requests


def make_register_payload() -> dict[str, str]:
    suffix = uuid.uuid4().hex[:12]
    return {
        "username": f"rules_e2e_{suffix}",
        "password": "correct horse battery staple",
        "email": f"rules_e2e_{suffix}@example.com",
        "given_name": "Rules",
        "surname": "Tester",
    }


def unique_name(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:12]}"


def register_session(stack_urls: dict[str, str]) -> requests.Session:
    session = requests.Session()
    response = session.post(
        f"{stack_urls['server_url']}/auth/register",
        json=make_register_payload(),
        timeout=5,
    )
    assert response.status_code == 200, response.text
    assert session.cookies.get("session_id"), "expected authenticated session cookie"
    assert session.cookies.get("csfr_"), "expected CSRF cookie for authenticated session"
    return session


def csrf_headers(session: requests.Session) -> dict[str, str]:
    token = session.cookies.get("csfr_")
    assert token, "expected CSRF cookie before issuing mutating request"
    return {"x-csrf-token": token}


def create_rule(
    session: requests.Session,
    stack_urls: dict[str, str],
    *,
    name: str,
    trigger: dict[str, object],
    script: str,
    description: str = "",
    enabled: bool = True,
) -> dict[str, object]:
    response = session.post(
        f"{stack_urls['server_url']}/rule",
        json={
            "name": name,
            "description": description,
            "trigger": trigger,
            "script": script,
            "enabled": enabled,
        },
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 201, response.text
    return response.json()


def get_rule(
    session: requests.Session,
    stack_urls: dict[str, str],
    rule_id: int,
) -> dict[str, object]:
    response = session.get(f"{stack_urls['server_url']}/rule/{rule_id}", timeout=5)
    assert response.status_code == 200, response.text
    return response.json()


def run_rule(
    session: requests.Session,
    stack_urls: dict[str, str],
    rule_id: int,
) -> dict[str, object]:
    response = session.post(
        f"{stack_urls['server_url']}/rule/{rule_id}/run",
        json={},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 200, response.text
    return response.json()


def wait_for_rule_run(
    session: requests.Session,
    stack_urls: dict[str, str],
    rule_id: int,
    *,
    timeout_s: float = 5.0,
) -> dict[str, object]:
    deadline = time.monotonic() + timeout_s
    latest_rule = get_rule(session, stack_urls, rule_id)
    while time.monotonic() < deadline:
        latest_rule = get_rule(session, stack_urls, rule_id)
        if latest_rule["last_run_at"] is not None:
            return latest_rule
        time.sleep(0.2)
    return latest_rule


def create_item(
    session: requests.Session,
    stack_urls: dict[str, str],
    *,
    title: str,
    content: str = "",
    attributes: dict[str, object] | None = None,
    types: list[str] | None = None,
) -> dict[str, object]:
    payload: dict[str, object] = {
        "title": title,
        "content": content,
    }
    if attributes is not None:
        payload["attributes"] = attributes
    if types is not None:
        payload["types"] = types

    response = session.post(
        f"{stack_urls['server_url']}/item",
        json=payload,
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 200, response.text
    return response.json()


def get_item(
    session: requests.Session,
    stack_urls: dict[str, str],
    item_id: int,
) -> dict[str, object]:
    response = session.get(f"{stack_urls['server_url']}/item/id/{item_id}", timeout=5)
    assert response.status_code == 200, response.text
    return response.json()


def get_item_by_title(
    session: requests.Session,
    stack_urls: dict[str, str],
    title: str,
) -> requests.Response:
    return session.get(
        f"{stack_urls['server_url']}/item/title/{quote(title, safe='')}",
        timeout=5,
    )


def get_comments_for_item(
    session: requests.Session,
    stack_urls: dict[str, str],
    item_id: int,
) -> list[dict[str, object]]:
    response = session.get(f"{stack_urls['server_url']}/comment/item/{item_id}", timeout=5)
    assert response.status_code == 200, response.text
    return response.json()


def create_item_type(
    session: requests.Session,
    stack_urls: dict[str, str],
    *,
    name: str,
) -> dict[str, object]:
    response = session.post(
        f"{stack_urls['server_url']}/item_type",
        json={
            "name": name,
            "description": "",
            "required_attributes": [],
        },
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 200, response.text
    return response.json()


def test_manual_rule_run_captures_output_and_persists_run_metadata(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("manual-output"),
        trigger={"kind": "manual"},
        script="""
zealot.notify("Rules are working!")
zealot.log("Second line")
""".strip(),
    )

    result = run_rule(session, stack_urls, int(rule["rule_id"]))

    assert result["rule_id"] == rule["rule_id"]
    assert result["success"] is True
    assert result["output"] == "Rules are working!\nSecond line"
    assert result["error"] is None
    assert isinstance(result["duration_ms"], int)
    assert result["duration_ms"] >= 0

    stored_rule = get_rule(session, stack_urls, int(rule["rule_id"]))
    assert stored_rule["last_run_at"] is not None
    assert stored_rule["last_output"] == "Rules are working!\nSecond line"
    assert stored_rule["last_error"] is None


def test_manual_rule_has_nil_event_context(stack_urls: dict[str, str]) -> None:
    session = register_session(stack_urls)
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("manual-event-nil"),
        trigger={"kind": "manual"},
        script="""
if zealot.event ~= nil then
    error("expected nil event for manual execution")
end
zealot.notify("event-is-nil")
""".strip(),
    )

    result = run_rule(session, stack_urls, int(rule["rule_id"]))

    assert result["success"] is True
    assert result["output"] == "event-is-nil"
    assert result["error"] is None


def test_manual_rule_can_update_items_and_add_comments(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    item = create_item(
        session,
        stack_urls,
        title=unique_name("rule-target"),
        content="Before rule run",
    )
    item_id = int(item["item_id"])
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("manual-side-effects"),
        trigger={"kind": "manual"},
        script=f"""
zealot.items.set_attribute({item_id}, "Qa Flag", "set by rule")
zealot.comments.add({item_id}, "Rule touched this item")
zealot.notify("updated")
""".strip(),
    )

    result = run_rule(session, stack_urls, int(rule["rule_id"]))

    assert result["success"] is True
    assert result["output"] == "updated"

    updated_item = get_item(session, stack_urls, item_id)
    comments = get_comments_for_item(session, stack_urls, item_id)

    assert updated_item["attributes"]["Qa Flag"] == "set by rule"
    assert any(comment["content"] == "Rule touched this item" for comment in comments)


def test_documented_items_create_signature_accepts_content_and_options(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    item_type = create_item_type(
        session,
        stack_urls,
        name=unique_name("rule-created-type"),
    )
    item_type_name = str(item_type["name"])
    title = unique_name("doc-create-item")
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("doc-create-signature"),
        trigger={"kind": "manual"},
        script=f"""
local item = zealot.items.create("{title}", "Body from docs", {{
    types = {{ "{item_type_name}" }},
}})
zealot.notify(item.title)
""".strip(),
    )

    result = run_rule(session, stack_urls, int(rule["rule_id"]))

    assert result["success"] is True, result
    assert result["output"] == title, result

    created_item_response = get_item_by_title(session, stack_urls, title)
    assert created_item_response.status_code == 200, created_item_response.text
    created_item = created_item_response.json()
    assert created_item["content"] == "Body from docs"
    assert any(type_ref["name"] == item_type_name for type_ref in created_item["types"])


def test_documented_get_by_title_returns_a_single_item_or_nil(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    title = unique_name("doc-get-by-title")
    create_item(session, stack_urls, title=title, content="Lookup target")
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("doc-get-by-title"),
        trigger={"kind": "manual"},
        script=f"""
local item = zealot.items.get_by_title("{title}")
if not item then
    error("expected item")
end
if item.title ~= "{title}" then
    error("expected direct item table from get_by_title")
end
zealot.notify(item.title)
""".strip(),
    )

    result = run_rule(session, stack_urls, int(rule["rule_id"]))

    assert result["success"] is True, result
    assert result["output"] == title, result
    assert result["error"] is None, result


def test_on_item_create_rule_runs_when_an_item_is_created(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name("on-item-create"),
        trigger={"kind": "on_item_create"},
        script='zealot.notify("fired")',
    )

    create_item(
        session,
        stack_urls,
        title=unique_name("trigger-source"),
        content="Should fire on_item_create",
    )

    stored_rule = wait_for_rule_run(session, stack_urls, int(rule["rule_id"]))

    assert stored_rule["last_run_at"] is not None, stored_rule
    assert stored_rule["last_output"] == "fired"
    assert stored_rule["last_error"] is None


def _event_rule(
    session: requests.Session,
    stack_urls: dict[str, str],
    kind: str,
    script: str,
    **trigger_extra: object,
) -> int:
    rule = create_rule(
        session,
        stack_urls,
        name=unique_name(kind),
        trigger={"kind": kind, **trigger_extra},
        script=script,
    )
    return int(rule["rule_id"])


def _rule_output(
    session: requests.Session, stack_urls: dict[str, str], rule_id: int
) -> str:
    rule = wait_for_rule_run(session, stack_urls, rule_id)
    assert rule["last_error"] is None, rule
    return str(rule["last_output"])


ITEM_PAYLOAD_SCRIPT = """
local e = zealot.event
local i = e.item
zealot.notify(table.concat({
    e.kind, e.item_id, i.id, i.title, i.attributes["Status"], i.types[1] or "-",
}, "|"))
"""


def test_documented_event_payloads_for_every_event_kind(
    stack_urls: dict[str, str],
) -> None:
    """Each documented zealot.event field arrives for each event kind."""
    session = register_session(stack_urls)
    base = stack_urls["server_url"]
    headers = csrf_headers(session)
    type_name = unique_name("PayloadType")
    create_item_type(session, stack_urls, name=type_name)

    create_rule_id = _event_rule(session, stack_urls, "on_item_create", ITEM_PAYLOAD_SCRIPT)
    update_rule_id = _event_rule(session, stack_urls, "on_item_update", ITEM_PAYLOAD_SCRIPT)
    assign_rule_id = _event_rule(
        session,
        stack_urls,
        "on_type_assign",
        'zealot.notify(zealot.event.kind .. "|" .. zealot.event.item_id .. "|" .. zealot.event.item.id'
        ' .. "|" .. zealot.event.item.title .. "|" .. zealot.event.type_name)',
    )
    unassign_rule_id = _event_rule(
        session,
        stack_urls,
        "on_type_unassign",
        'zealot.notify(zealot.event.kind .. "|" .. zealot.event.item.id .. "|" .. zealot.event.type_name)',
    )
    attr_rule_id = _event_rule(
        session,
        stack_urls,
        "on_attribute_set",
        """
local e = zealot.event
zealot.notify(table.concat({
    e.kind, e.item_id, e.item.id, e.attribute_key,
    tostring(e.old_value), tostring(e.new_value), tostring(e.item.attributes["Status"]),
}, "|"))
""",
        attribute_key="Status",
    )
    comment_rule_id = _event_rule(
        session,
        stack_urls,
        "on_comment_add",
        'local e = zealot.event; zealot.notify(table.concat({e.kind, e.item_id, e.comment_id, e.content, e.timestamp ~= nil and "ts" or "nots"}, "|"))',
    )
    delete_rule_id = _event_rule(
        session,
        stack_urls,
        "on_item_delete",
        'zealot.notify(zealot.event.kind .. "|" .. zealot.event.item_id)',
    )

    title = unique_name("payload-item")
    item = create_item(
        session, stack_urls, title=title, attributes={"Status": "To Do"}
    )
    item_id = int(item["item_id"])

    assert _rule_output(session, stack_urls, create_rule_id) == (
        f"on_item_create|{item_id}|{item_id}|{title}|To Do|-"
    )

    response = session.patch(
        f"{base}/item/{item_id}", json={"item_id": item_id, "content": "changed"}, headers=headers, timeout=5
    )
    assert response.status_code == 200, response.text
    assert _rule_output(session, stack_urls, update_rule_id) == (
        f"on_item_update|{item_id}|{item_id}|{title}|To Do|-"
    )

    response = session.post(
        f"{base}/item/{item_id}/assign_type/{quote(type_name, safe='')}",
        json={},
        headers=headers,
        timeout=5,
    )
    assert response.status_code == 200, response.text
    assert _rule_output(session, stack_urls, assign_rule_id) == (
        f"on_type_assign|{item_id}|{item_id}|{title}|{type_name}"
    )

    response = session.patch(
        f"{base}/item/{item_id}/attr", json={"Status": "Complete"}, headers=headers, timeout=5
    )
    assert response.status_code == 200, response.text
    assert _rule_output(session, stack_urls, attr_rule_id) == (
        f"on_attribute_set|{item_id}|{item_id}|Status|To Do|Complete|Complete"
    )

    response = session.delete(f"{base}/item/{item_id}/attr/Status", headers=headers, timeout=5)
    assert response.status_code == 200, response.text
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        removed_out = _rule_output(session, stack_urls, attr_rule_id)
        if "nil" in removed_out:
            break
        time.sleep(0.2)
    assert removed_out == f"on_attribute_set|{item_id}|{item_id}|Status|Complete|nil|nil"

    response = session.post(
        f"{base}/comment",
        json={"item_id": item_id, "timestamp": "2026-10-01 10:00:00", "content": "hello"},
        headers=headers,
        timeout=5,
    )
    assert response.status_code == 200, response.text
    comment_out = _rule_output(session, stack_urls, comment_rule_id).split("|")
    assert comment_out[0] == "on_comment_add"
    assert comment_out[1] == str(item_id)
    assert comment_out[3:] == ["hello", "ts"]

    response = session.delete(
        f"{base}/item/{item_id}/assign_type/{quote(type_name, safe='')}",
        headers=headers,
        timeout=5,
    )
    assert response.status_code == 200, response.text
    assert _rule_output(session, stack_urls, unassign_rule_id) == (
        f"on_type_unassign|{item_id}|{type_name}"
    )

    response = session.delete(f"{base}/item/{item_id}", headers=headers, timeout=5)
    assert response.status_code in (200, 204), response.text
    assert _rule_output(session, stack_urls, delete_rule_id) == (
        f"on_item_delete|{item_id}"
    )


def test_attributes_set_at_creation_fire_on_item_create_not_on_attribute_set(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    create_rule_id = _event_rule(
        session,
        stack_urls,
        "on_item_create",
        'zealot.notify(zealot.event.item.attributes["Status"])',
    )
    attr_rule_id = _event_rule(
        session, stack_urls, "on_attribute_set", 'zealot.notify("attr")'
    )
    create_item(
        session,
        stack_urls,
        title=unique_name("created-complete"),
        attributes={"Status": "Complete"},
    )
    assert _rule_output(session, stack_urls, create_rule_id) == "Complete"
    time.sleep(1)
    attr_rule = get_rule(session, stack_urls, attr_rule_id)
    assert attr_rule["last_run_at"] is None, attr_rule

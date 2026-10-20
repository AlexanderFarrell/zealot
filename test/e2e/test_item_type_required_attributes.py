from __future__ import annotations

import uuid

import requests


def make_register_payload() -> dict[str, str]:
    suffix = uuid.uuid4().hex[:12]
    return {
        "username": f"typereq_e2e_{suffix}",
        "password": "correct horse battery staple",
        "email": f"typereq_e2e_{suffix}@example.com",
        "given_name": "Type",
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


def post_item_type(
    session: requests.Session, stack_urls: dict[str, str], name: str, required: list[str]
) -> requests.Response:
    return session.post(
        f"{stack_urls['server_url']}/item_type",
        json={"name": name, "description": "", "required_attributes": required},
        headers=csrf_headers(session),
        timeout=5,
    )


def get_item_type_by_name(
    session: requests.Session, stack_urls: dict[str, str], name: str
) -> requests.Response:
    return session.get(f"{stack_urls['server_url']}/item_type/name/{name}", timeout=5)


def test_create_item_type_rejects_unknown_required_attributes(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    name = unique_name("Unknown Required")
    missing = unique_name("Missing Kind")

    response = post_item_type(session, stack_urls, name, ["Priority", missing])
    assert response.status_code == 400, response.text
    assert missing in response.text
    assert "'Priority'" not in response.text

    # Nothing was stored.
    assert get_item_type_by_name(session, stack_urls, name).status_code == 404


def test_update_item_type_rejects_unknown_required_attributes(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    name = unique_name("Update Required")
    missing = unique_name("Missing Kind")
    created = post_item_type(session, stack_urls, name, ["Priority"])
    assert created.status_code == 200, created.text
    type_id = created.json()["type_id"]

    response = session.patch(
        f"{stack_urls['server_url']}/item_type/{type_id}",
        json={"type_id": type_id, "required_attributes": ["Priority", missing]},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 400, response.text
    assert missing in response.text

    stored = get_item_type_by_name(session, stack_urls, name)
    assert stored.status_code == 200, stored.text
    assert stored.json()["required_attributes"] == ["Priority"]


def test_item_type_with_a_user_attribute_kind_reports_what_was_stored(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    key = unique_name("Icon Kind")
    kind = session.post(
        f"{stack_urls['server_url']}/attribute",
        json={"key": key, "description": "", "base_type": "text", "config": {}},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert kind.status_code == 200, kind.text

    name = unique_name("Known Required")
    created = post_item_type(session, stack_urls, name, [key])
    assert created.status_code == 200, created.text
    assert created.json()["required_attributes"] == [key]

    stored = get_item_type_by_name(session, stack_urls, name)
    assert stored.json()["required_attributes"] == [key]

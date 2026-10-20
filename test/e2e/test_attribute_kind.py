from __future__ import annotations

import uuid

import requests


def make_register_payload() -> dict[str, str]:
    suffix = uuid.uuid4().hex[:12]
    return {
        "username": f"attrkind_e2e_{suffix}",
        "password": "correct horse battery staple",
        "email": f"attrkind_e2e_{suffix}@example.com",
        "given_name": "Attribute",
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


def post_attribute_kind(
    session: requests.Session, stack_urls: dict[str, str], key: str
) -> requests.Response:
    return session.post(
        f"{stack_urls['server_url']}/attribute",
        json={"key": key, "description": "", "base_type": "text", "config": {}},
        headers=csrf_headers(session),
        timeout=5,
    )


def test_duplicate_attribute_kind_key_returns_conflict_naming_the_key(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    key = unique_name("Dup Kind")

    first = post_attribute_kind(session, stack_urls, key)
    assert first.status_code == 200, first.text

    second = post_attribute_kind(session, stack_urls, key)
    assert second.status_code == 409, second.text
    assert key in second.text


def test_renaming_attribute_kind_onto_an_existing_key_returns_conflict(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    taken = unique_name("Taken Kind")
    other = unique_name("Other Kind")
    assert post_attribute_kind(session, stack_urls, taken).status_code == 200
    created = post_attribute_kind(session, stack_urls, other)
    assert created.status_code == 200, created.text

    response = session.patch(
        f"{stack_urls['server_url']}/attribute/id/{created.json()['kind_id']}",
        json={"kind_id": created.json()["kind_id"], "key": taken},
        headers=csrf_headers(session),
        timeout=5,
    )
    assert response.status_code == 409, response.text
    assert taken in response.text

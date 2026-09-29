from __future__ import annotations

import uuid

import requests


def make_register_payload() -> dict[str, str]:
    suffix = uuid.uuid4().hex[:12]
    return {
        "username": f"e2e_{suffix}",
        "password": "correct horse battery staple",
        "email": f"e2e_{suffix}@example.com",
        "given_name": "E2E",
        "surname": "Tester",
    }


def assert_account_payload_matches(
    response_json: dict[str, str | int],
    payload: dict[str, str],
) -> None:
    assert response_json["username"] == payload["username"]
    assert response_json["email"] == payload["email"]
    assert response_json["given_name"] == payload["given_name"]
    assert response_json["surname"] == payload["surname"]
    assert isinstance(response_json["account_id"], int)


def test_auth_status_endpoints_reject_anonymous_requests(
    stack_urls: dict[str, str],
) -> None:
    root_response = requests.get(f"{stack_urls['server_url']}/auth", timeout=5)
    status_response = requests.get(f"{stack_urls['server_url']}/auth/is_logged_in", timeout=5)

    assert root_response.status_code == 401
    assert root_response.text.strip() == "Unauthorized"
    assert status_response.status_code == 401
    assert status_response.text.strip() == "Unauthorized"


def test_register_creates_authenticated_session(
    stack_urls: dict[str, str],
) -> None:
    payload = make_register_payload()
    session = requests.Session()

    response = session.post(
        f"{stack_urls['server_url']}/auth/register",
        json=payload,
        timeout=5,
    )

    assert response.status_code == 200
    response_json = response.json()
    assert_account_payload_matches(response_json, payload)
    assert session.cookies.get("session_id"), "expected register to create a session cookie"

    is_logged_in_response = session.get(
        f"{stack_urls['server_url']}/auth/is_logged_in",
        timeout=5,
    )

    assert is_logged_in_response.status_code == 200
    assert_account_payload_matches(is_logged_in_response.json(), payload)


def test_login_returns_account_and_session_cookie(
    stack_urls: dict[str, str],
) -> None:
    payload = make_register_payload()

    register_session = requests.Session()
    register_response = register_session.post(
        f"{stack_urls['server_url']}/auth/register",
        json=payload,
        timeout=5,
    )
    assert register_response.status_code == 200

    login_session = requests.Session()
    login_response = login_session.post(
        f"{stack_urls['server_url']}/auth/login",
        json={
            "username": payload["username"],
            "password": payload["password"],
        },
        timeout=5,
    )

    assert login_response.status_code == 200
    assert_account_payload_matches(login_response.json(), payload)
    assert login_session.cookies.get("session_id"), "expected login to set a session cookie"

    is_logged_in_response = login_session.get(
        f"{stack_urls['server_url']}/auth/is_logged_in",
        timeout=5,
    )

    assert is_logged_in_response.status_code == 200
    assert_account_payload_matches(is_logged_in_response.json(), payload)


def test_logout_clears_authenticated_session(
    stack_urls: dict[str, str],
) -> None:
    payload = make_register_payload()
    session = requests.Session()

    register_response = session.post(
        f"{stack_urls['server_url']}/auth/register",
        json=payload,
        timeout=5,
    )
    assert register_response.status_code == 200

    logout_response = session.post(
        f"{stack_urls['server_url']}/auth/logout",
        timeout=5,
    )

    assert logout_response.status_code == 200

    is_logged_in_response = session.get(
        f"{stack_urls['server_url']}/auth/is_logged_in",
        timeout=5,
    )

    assert is_logged_in_response.status_code == 401
    assert is_logged_in_response.text.strip() == "Unauthorized"

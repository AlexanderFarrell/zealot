from __future__ import annotations

import uuid

import requests


def make_register_payload() -> dict[str, str]:
    suffix = uuid.uuid4().hex[:12]
    return {
        "username": f"filter_e2e_{suffix}",
        "password": "correct horse battery staple",
        "email": f"filter_e2e_{suffix}@example.com",
        "given_name": "Filter",
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


def create_item(
    session: requests.Session,
    stack_urls: dict[str, str],
    *,
    title: str,
    attributes: dict[str, object],
) -> dict[str, object]:
    response = session.post(
        f"{stack_urls['server_url']}/item",
        json={"title": title, "content": "", "attributes": attributes},
        headers=csrf_headers(session),
        timeout=10,
    )
    assert response.status_code == 200, response.text
    return response.json()


def filter_items(
    session: requests.Session,
    stack_urls: dict[str, str],
    *,
    filters: list[dict[str, object]],
    limit: int | None = None,
    offset: int | None = None,
) -> list[dict[str, object]]:
    body: dict[str, object] = {"filters": filters}
    if limit is not None:
        body["limit"] = limit
    if offset is not None:
        body["offset"] = offset

    response = session.post(
        f"{stack_urls['server_url']}/item/filter",
        json=body,
        headers=csrf_headers(session),
        timeout=10,
    )
    assert response.status_code == 200, response.text
    return response.json()


def test_item_filter_paginates_broad_filters_and_supports_ilike(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    bucket_key = unique_name("Z118 Bucket")
    text_key = unique_name("Z118 Text")
    title_prefix = unique_name("Z118 Filtered")

    for index in range(105):
        create_item(
            session,
            stack_urls,
            title=f"{title_prefix} {index:03d}",
            attributes={
                bucket_key: "common",
                text_key: f"Needle Payload {index:03d}",
            },
        )
    create_item(
        session,
        stack_urls,
        title=f"{title_prefix} other",
        attributes={bucket_key: "other", text_key: "Different Payload"},
    )

    specific = filter_items(
        session,
        stack_urls,
        filters=[{"key": bucket_key, "op": "eq", "value": "other"}],
    )
    assert [item["title"] for item in specific] == [f"{title_prefix} other"]

    default_page = filter_items(
        session,
        stack_urls,
        filters=[{"key": bucket_key, "op": "eq", "value": "common"}],
    )
    assert len(default_page) == 50
    assert default_page[0]["title"] == f"{title_prefix} 000"
    assert default_page[-1]["title"] == f"{title_prefix} 049"

    offset_page = filter_items(
        session,
        stack_urls,
        filters=[{"key": bucket_key, "op": "eq", "value": "common"}],
        limit=10,
        offset=50,
    )
    assert [item["title"] for item in offset_page] == [
        f"{title_prefix} {index:03d}" for index in range(50, 60)
    ]

    capped_page = filter_items(
        session,
        stack_urls,
        filters=[{"key": bucket_key, "op": "eq", "value": "common"}],
        limit=500,
    )
    assert len(capped_page) == 100
    assert capped_page[-1]["title"] == f"{title_prefix} 099"

    ilike_matches = filter_items(
        session,
        stack_urls,
        filters=[{"key": text_key, "op": "ilike", "value": "payload 010"}],
        limit=5,
    )
    assert [item["title"] for item in ilike_matches] == [f"{title_prefix} 010"]


def test_item_filter_supports_all_list_mode_and_array_values(
    stack_urls: dict[str, str],
) -> None:
    session = register_session(stack_urls)
    title_prefix = unique_name("Z118 Parent Filter")

    parent_a = create_item(
        session,
        stack_urls,
        title=f"{title_prefix} parent A",
        attributes={},
    )
    parent_b = create_item(
        session,
        stack_urls,
        title=f"{title_prefix} parent B",
        attributes={},
    )
    child_both = create_item(
        session,
        stack_urls,
        title=f"{title_prefix} child both",
        attributes={"Parent": [parent_a["item_id"], parent_b["item_id"]]},
    )
    child_one = create_item(
        session,
        stack_urls,
        title=f"{title_prefix} child one",
        attributes={"Parent": [parent_a["item_id"]]},
    )

    scalar_all = filter_items(
        session,
        stack_urls,
        filters=[
            {
                "key": "Parent",
                "op": "eq",
                "value": parent_a["item_id"],
                "list_mode": "all",
            }
        ],
    )
    assert {item["item_id"] for item in scalar_all} == {
        child_both["item_id"],
        child_one["item_id"],
    }

    array_any = filter_items(
        session,
        stack_urls,
        filters=[
            {
                "key": "Parent",
                "op": "eq",
                "value": [parent_a["item_id"], parent_b["item_id"]],
                "list_mode": "any",
            }
        ],
    )
    assert {item["item_id"] for item in array_any} == {
        child_both["item_id"],
        child_one["item_id"],
    }

    array_all = filter_items(
        session,
        stack_urls,
        filters=[
            {
                "key": "Parent",
                "op": "eq",
                "value": [parent_a["item_id"], parent_b["item_id"]],
                "list_mode": "all",
            }
        ],
    )
    assert [item["item_id"] for item in array_all] == [child_both["item_id"]]

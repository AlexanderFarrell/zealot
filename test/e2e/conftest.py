from __future__ import annotations

import os

import pytest


SERVER_URL = os.getenv("ZEALOT_E2E_SERVER_URL", "http://127.0.0.1:18456")
WEB_URL = os.getenv("ZEALOT_E2E_WEB_URL", "http://127.0.0.1:18080")


@pytest.fixture(scope="session")
def stack_urls() -> dict[str, str]:
    return {
        "server_url": SERVER_URL,
        "web_url": WEB_URL,
    }

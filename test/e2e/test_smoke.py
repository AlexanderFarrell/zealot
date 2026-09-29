from __future__ import annotations

import re
from urllib.parse import urljoin

import requests


def test_server_health_endpoints(stack_urls: dict[str, str]) -> None:
    health_response = requests.get(f"{stack_urls['server_url']}/health", timeout=5)
    ready_response = requests.get(f"{stack_urls['server_url']}/health/ready", timeout=5)

    assert health_response.status_code == 200
    assert health_response.text.strip() == "ok"
    assert ready_response.status_code == 200
    assert ready_response.text.strip() == "ready"


def test_web_root_serves_built_shell(stack_urls: dict[str, str]) -> None:
    response = requests.get(f"{stack_urls['web_url']}/", timeout=5)

    assert response.status_code == 200
    assert "<!doctype html>" in response.text.lower()
    assert '<div id="app"></div>' in response.text


def test_web_proxies_api_health(stack_urls: dict[str, str]) -> None:
    health_response = requests.get(f"{stack_urls['web_url']}/api/health", timeout=5)
    ready_response = requests.get(f"{stack_urls['web_url']}/api/health/ready", timeout=5)

    assert health_response.status_code == 200
    assert health_response.text.strip() == "ok"
    assert ready_response.status_code == 200
    assert ready_response.text.strip() == "ready"


def test_web_serves_built_asset(stack_urls: dict[str, str]) -> None:
    html_response = requests.get(f"{stack_urls['web_url']}/", timeout=5)
    asset_paths = re.findall(r'(?:src|href)="([^"]*assets/[^"]+)"', html_response.text)

    assert asset_paths, "expected built asset references in the web root HTML"

    asset_url = urljoin(f"{stack_urls['web_url']}/", asset_paths[0])
    asset_response = requests.get(asset_url, timeout=5)

    assert asset_response.status_code == 200
    assert asset_response.content

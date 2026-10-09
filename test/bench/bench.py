#!/usr/bin/env python3
"""Z221 benchmark runner. See README.md.

  bench.py prepare --url URL                 register the benchmark account, kinds and types
  bench.py check   --url URL --scale N       verify the seeded database
  bench.py run     --url URL --scale N ...   measure operations and write JSON + Markdown

The seeded data model (ids, titles, types, links) is defined by seed.*.sql; the
helper functions here must stay in step with it.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import random
import statistics
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor

import requests

USER = "bench"
PASSWORD = "correct horse battery staple"
EMAIL = "bench@example.com"

TYPE_NAMES = ["Ticket", "Topic", "Book", "Word", "Recipe", "Project", "Epic", "Document", "Meeting", "Repo", "Game"]
WORDS = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta"]

# Abort rule (approved spec): stop an operation after this many consecutive samples
# above SLOW_S seconds, or after OP_BUDGET_S seconds in total.
SLOW_S = 30.0
SLOW_STREAK = 3
OP_BUDGET_S = 600.0
REQUEST_TIMEOUT_S = 60


def title_of(i: int) -> str:
    return f"Item {i:07d} {WORDS[i % 8]}"


class Client:
    def __init__(self, url: str):
        self.url = url.rstrip("/")
        self.s = requests.Session()
        self.h: dict[str, str] = {}

    def login(self) -> None:
        r = self.s.post(f"{self.url}/auth/login", json={"username": USER, "password": PASSWORD}, timeout=30)
        if r.status_code != 200:
            r = self.s.post(
                f"{self.url}/auth/register",
                json={"username": USER, "password": PASSWORD, "email": EMAIL, "given_name": "Bench", "surname": "Mark"},
                timeout=30,
            )
        r.raise_for_status()
        self.h = {"x-csrf-token": self.s.cookies.get("csfr_") or ""}

    def req(self, method: str, path: str, **kw) -> requests.Response:
        return self.s.request(method, self.url + path, headers=self.h, timeout=REQUEST_TIMEOUT_S, **kw)


def cmd_prepare(a) -> None:
    c = Client(a.url)
    c.login()
    kinds = {k["key"] for k in c.req("GET", "/attribute").json()}
    wanted = [
        ("Description", "text", {}),
        ("TicketID", "text", {}),
        ("Rating", "integer", {}),
        ("Topic", "list", {"list_type": "item"}),
    ]
    for key, base, cfg in wanted:
        if key not in kinds:
            r = c.req("POST", "/attribute", json={"key": key, "description": "bench", "base_type": base, "config": cfg})
            r.raise_for_status()
    have = {t["name"] for t in c.req("GET", "/item_type").json()}
    required = {"Ticket": ["Status", "Priority", "TicketID"], "Project": ["Status"], "Epic": ["Status", "Priority"], "Book": ["Rating"]}
    for name in TYPE_NAMES:
        if name not in have:
            r = c.req("POST", "/item_type", json={"name": name, "description": "bench", "required_attributes": required.get(name, [])})
            r.raise_for_status()
    print("prepared")


def cmd_check(a) -> None:
    c = Client(a.url)
    c.login()
    n = a.scale
    summary = {t["name"]: t["item_count"] for t in c.req("GET", "/item_type/summary").json()}
    problems = []
    expected_ticket = len([i for i in range(1, n + 1) if i % 12 == 0]) if n <= 20000 else n // 12
    if summary.get("Ticket") != expected_ticket:
        problems.append(f"Ticket count {summary.get('Ticket')} != expected {expected_ticket}")
    rng = random.Random(1)
    for i in [1, 2, n] + [rng.randint(3, n - 1) for _ in range(30)]:
        r = c.req("GET", f"/item/id/{i}")
        if r.status_code != 200:
            problems.append(f"GET item {i}: {r.status_code}")
            continue
        it = r.json()
        if it["title"] != title_of(i):
            problems.append(f"item {i} title {it['title']!r}")
        parents = it["attributes"].get("Parent") or []
        link_parents = sorted(l["other_item_id"] for l in it["links"] if l["relationship"] == "parent")
        if sorted(parents) != link_parents:
            problems.append(f"item {i} Parent attr {parents} != parent links {link_parents}")
        if i % 9 == 0 and not any(l["relationship"] == "wikilink" for l in it["links"]):
            problems.append(f"item {i} missing wikilink row")
        if i % 12 == 11 and it["types"]:
            problems.append(f"item {i} should be untyped")
    r = c.req("POST", "/item/query", json={"types": ["Ticket"], "limit": 1})
    if r.status_code != 200 or r.json().get("total") != expected_ticket:
        problems.append(f"/item/query Ticket total {r.json().get('total') if r.ok else r.status_code} != {expected_ticket}")
    if problems:
        print("CHECK FAILED")
        for p in problems:
            print(" -", p)
        sys.exit(1)
    print(f"check ok: {n} items, {sum(summary.values())} type memberships, Ticket={expected_ticket}")


# ---------------------------------------------------------------- operations

def typed_id(rng: random.Random, n: int) -> int:
    while True:
        i = rng.randint(2, n)
        if i % 12 != 11:
            return i


def untyped_id(rng: random.Random, n: int) -> int:
    m = (n - 11) // 12
    return 12 * rng.randint(0, max(m, 0)) + 11 if m > 0 else 11


def build_ops(n: int, run_id: str):
    """name -> (heavy, callable(client, rng, k) -> response)."""
    ops = {}

    def get_item(c, rng, k):
        return c.req("GET", f"/item/id/{rng.randint(1, n)}")

    def children(c, rng, k):
        return c.req("GET", f"/item/children/{rng.randint(1, max(2, n // 10))}")

    def content_edit(c, rng, k):
        i = typed_id(rng, n)
        return c.req("PATCH", f"/item/{i}", json={"item_id": i, "content": ("benchmark edit %s %d " % (run_id, k)) * 40})

    def content_edit_wikilink(c, rng, k):
        i = typed_id(rng, n)
        j = rng.randint(1, n)
        return c.req("PATCH", f"/item/{i}", json={"item_id": i, "content": f"edit {run_id} {k} see [[{title_of(j)}]]"})

    def attr_scalar(c, rng, k):
        i = typed_id(rng, n)
        return c.req("PATCH", f"/item/{i}/attr", json={"Priority": 1 + rng.randint(0, 9)})

    def attr_parent(c, rng, k):
        i = rng.randint(3, n)
        return c.req("PATCH", f"/item/{i}/attr", json={"Parent": [rng.randint(1, i - 1)]})

    def type_assign(c, rng, k):
        return c.req("POST", f"/item/{untyped_id(rng, n)}/assign_type/Word")

    def filter_type_attr(c, rng, k):
        return c.req("POST", "/item/query", json={
            "types": ["Ticket"], "limit": 50,
            "filters": [{"key": "Status", "op": "eq", "value": ["To Do", "Working", "Complete", "Hold", "Specify"][k % 5]}],
        })

    def filter_attr_sorted(c, rng, k):
        return c.req("POST", "/item/query", json={
            "types": ["Book"], "limit": 50, "sort": {"key": "Rating", "direction": "desc"},
            "filters": [{"key": "Rating", "op": "gte", "value": 4}],
        })

    def under(c, rng, k):
        return c.req("POST", "/item/query", json={"under": rng.randint(1, max(2, n // 100)), "limit": 50})

    def type_screen(c, rng, k):
        return c.req("GET", "/item", params={"type": ["Ticket", "Book", "Project"][k % 3]})

    def type_summary(c, rng, k):
        return c.req("GET", "/item_type/summary")

    def create_item(c, rng, k):
        return c.req("POST", "/item", json={"title": f"Created {run_id} {k} {rng.random():.9f}", "content": "new"})

    def title_edit(c, rng, k):
        i = typed_id(rng, n)
        return c.req("PATCH", f"/item/{i}", json={"item_id": i, "title": f"Renamed {run_id} {i} {k}"})

    # (name, heavy, fn). title_edit is last: it is the operation expected to hit the abort rule,
    # and it can leave the server busy for a long time afterwards.
    return [
        ("get_item", False, get_item),
        ("children", False, children),
        ("filter_type_attr", False, filter_type_attr),
        ("filter_attr_sorted", False, filter_attr_sorted),
        ("query_under", False, under),
        ("type_summary", False, type_summary),
        ("type_screen_list", True, type_screen),
        ("create_item", False, create_item),
        ("attr_set_scalar", False, attr_scalar),
        ("attr_set_parent", False, attr_parent),
        ("type_assign", False, type_assign),
        ("content_edit", False, content_edit),
        ("content_edit_wikilink", False, content_edit_wikilink),
        ("title_edit", False, title_edit),
    ]


def pct(sorted_ms: list[float], p: float) -> float:
    if not sorted_ms:
        return float("nan")
    return sorted_ms[min(len(sorted_ms) - 1, int(p * len(sorted_ms)))]


def run_op(name, fn, url, seed, samples, warmup, clients):
    lat: list[float] = []
    errors: list[str] = []
    state = {"streak": 0, "abort": None, "done": 0}
    lock = threading.Lock()
    start = time.time()

    def worker(widx, count, with_warmup):
        c = Client(url)
        c.login()
        rng = random.Random(seed * 1000 + widx)
        if with_warmup:
            for k in range(warmup):
                try:
                    fn(c, rng, 100000 + k)
                except Exception:
                    pass
        for k in range(count):
            if state["abort"] or time.time() - start > OP_BUDGET_S:
                with lock:
                    state["abort"] = state["abort"] or f"budget {OP_BUDGET_S:.0f}s exceeded"
                return
            t = time.time()
            try:
                r = fn(c, rng, widx * 100000 + k)
                dt = time.time() - t
                ok = 200 <= r.status_code < 300
                msg = None if ok else f"HTTP {r.status_code}: {r.text[:120]}"
            except requests.exceptions.RequestException as e:
                dt = time.time() - t
                ok, msg = False, f"{type(e).__name__}: {str(e)[:100]}"
            with lock:
                state["done"] += 1
                if ok:
                    lat.append(dt * 1000)
                else:
                    errors.append(msg)
                if dt > SLOW_S:
                    state["streak"] += 1
                    if state["streak"] >= SLOW_STREAK:
                        state["abort"] = f"{SLOW_STREAK} consecutive samples over {SLOW_S:.0f}s"
                else:
                    state["streak"] = 0

    per = max(1, samples // clients)
    if clients == 1:
        worker(0, samples, True)
    else:
        with ThreadPoolExecutor(max_workers=clients) as ex:
            list(ex.map(lambda w: worker(w, per, True), range(clients)))
    wall = time.time() - start
    s = sorted(lat)
    return {
        "op": name, "clients": clients, "ok": len(s), "errors": len(errors),
        "first_error": errors[0] if errors else None,
        "p50_ms": round(pct(s, .5), 1), "p95_ms": round(pct(s, .95), 1), "p99_ms": round(pct(s, .99), 1),
        "max_ms": round(s[-1], 1) if s else None, "mean_ms": round(statistics.fmean(s), 1) if s else None,
        "wall_s": round(wall, 1), "abort": state["abort"],
    }


def host_info() -> dict:
    def sh(cmd):
        try:
            return subprocess.check_output(cmd, shell=True, text=True, stderr=subprocess.DEVNULL).strip()
        except Exception:
            return ""
    return {
        "platform": platform.platform(),
        "cpu": sh("sysctl -n machdep.cpu.brand_string") or platform.processor(),
        "cores": os.cpu_count(),
        "memory_gb": round(int(sh("sysctl -n hw.memsize") or 0) / 2**30, 1) or None,
        "git_commit": sh("git rev-parse --short HEAD"),
        "rustc": sh("rustc --version"),
    }


def cmd_run(a) -> None:
    run_id = time.strftime("%H%M%S")
    ops = build_ops(a.scale, run_id)
    if a.only:
        ops = [o for o in ops if o[0] in a.only.split(",")]
    results = []
    for name, heavy, fn in ops:
        samples = a.heavy_samples if heavy else a.samples
        print(f"[{name}] n={samples} ...", flush=True)
        r = run_op(name, fn, a.url, a.seed, samples, 0 if heavy else (2 if name == "title_edit" else a.warmup), 1)
        results.append(r)
        print(f"   p50 {r['p50_ms']} p95 {r['p95_ms']} max {r['max_ms']} errors {r['errors']} {r['abort'] or ''}", flush=True)
    concurrent = []
    if a.clients > 1:
        skip = {r["op"] for r in results if r["abort"] or not r["ok"]}
        for name, heavy, fn in ops:
            if name in skip:
                continue
            if name in ("content_edit", "attr_set_scalar", "attr_set_parent", "filter_type_attr", "title_edit"):
                r = run_op(name, fn, a.url, a.seed + 7, a.samples, 2 if name == "title_edit" else a.warmup, a.clients)
                concurrent.append(r)
                print(f"[{name} x{a.clients}] p50 {r['p50_ms']} p95 {r['p95_ms']} max {r['max_ms']} errors {r['errors']} {r['abort'] or ''}", flush=True)
    out = {"meta": {"scale": a.scale, "db": a.db, "url": a.url, "samples": a.samples, "seed": a.seed,
                    "when": time.strftime("%Y-%m-%d %H:%M:%S"), **host_info()},
           "single": results, "concurrent": concurrent}
    if a.out:
        os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
        with open(a.out, "w") as f:
            json.dump(out, f, indent=2)
    print(markdown(out))


def markdown(out: dict) -> str:
    m = out["meta"]
    lines = [f"### {m['db']} at {m['scale']:,} items ({m['when']}, {m['git_commit']})", "",
             "| operation | clients | ok | errors | p50 ms | p95 ms | p99 ms | max ms | note |",
             "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for r in out["single"] + out["concurrent"]:
        note = r["abort"] or (r["first_error"] or "")
        lines.append(f"| {r['op']} | {r['clients']} | {r['ok']} | {r['errors']} | {r['p50_ms']} | {r['p95_ms']} | {r['p99_ms']} | {r['max_ms']} | {note} |")
    return "\n".join(lines)


def main() -> None:
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)
    for name in ("prepare", "check", "run"):
        sp = sub.add_parser(name)
        sp.add_argument("--url", default="http://127.0.0.1:47213")
        if name != "prepare":
            sp.add_argument("--scale", type=int, required=True)
    r = sub.choices["run"]
    r.add_argument("--db", default="sqlite")
    r.add_argument("--samples", type=int, default=200)
    r.add_argument("--heavy-samples", type=int, default=10)
    r.add_argument("--warmup", type=int, default=20)
    r.add_argument("--clients", type=int, default=4)
    r.add_argument("--seed", type=int, default=42)
    r.add_argument("--only", default="")
    r.add_argument("--out", default="")
    a = p.parse_args()
    {"prepare": cmd_prepare, "check": cmd_check, "run": cmd_run}[a.cmd](a)


if __name__ == "__main__":
    main()

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Events } from "../logic/events";
import { delete_req, get_json, LazyData, post_json, SetApiKey, withCsrf } from "./api_helper";

type Call = { url: string; init: RequestInit };
let calls: Call[];
let reply: () => Response;

beforeEach(() => {
    calls = [];
    reply = () => new Response(JSON.stringify({ ok: true }), { status: 200 });
    vi.stubGlobal("fetch", vi.fn(async (url: string, init: RequestInit) => {
        calls.push({ url, init });
        return reply();
    }));
    vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    SetApiKey(null);
    document.cookie = "csfr_=; expires=Thu, 01 Jan 1970 00:00:00 GMT";
});

describe("requests", () => {
    it("sends GET without a CSRF round trip", async () => {
        expect(await get_json("/x")).toEqual({ ok: true });
        expect(calls).toHaveLength(1);
        expect(calls[0]?.init.method).toBe("GET");
        expect(calls[0]?.init.credentials).toBe("include");
    });

    it("fetches a CSRF cookie before a write when none is set", async () => {
        await post_json("/x", { a: 1 });
        expect(calls.map(c => c.init.method)).toEqual(["GET", "POST"]);
        expect(calls[1]?.init.body).toBe(JSON.stringify({ a: 1 }));
    });

    it("sends the CSRF token header when the cookie exists", async () => {
        document.cookie = "csfr_=tok%20en";
        await post_json("/x", {});
        expect(calls).toHaveLength(1);
        expect((calls[0]?.init.headers as Record<string, string>)["X-Csrf-Token"]).toBe("tok en");
        expect(withCsrf({ A: "b" })).toEqual({ A: "b", "X-Csrf-Token": "tok en" });
    });

    it("authenticates with an API key and skips CSRF", async () => {
        SetApiKey("secret");
        await delete_req("/x");
        expect(calls).toHaveLength(1);
        expect((calls[0]?.init.headers as Record<string, string>)["X-Api-Key"]).toBe("secret");
    });
});

describe("errors", () => {
    it("throws with the status and the caller's message", async () => {
        reply = () => new Response("", { status: 500 });
        await expect(get_json("/x", "Could not load")).rejects.toThrow("Could not load (status 500)");
    });

    it("signals a login redirect on 401", async () => {
        reply = () => new Response("", { status: 401 });
        const seen = vi.fn();
        Events.on("to_auth", seen);
        await expect(get_json("/x")).rejects.toThrow(/401/);
        Events.off("to_auth", seen);
        expect(seen).toHaveBeenCalledOnce();
    });

    it("logs only the status, never the url or body", async () => {
        reply = () => new Response("secret body", { status: 403 });
        await expect(get_json("/private/thing")).rejects.toThrow();
        const logged = vi.mocked(console.error).mock.calls.flat().join(" ");
        expect(logged).toContain("403");
        expect(logged).not.toContain("secret body");
    });
});

describe("LazyData", () => {
    it("caches the value", async () => {
        const source = vi.fn(async () => 1);
        const lazy = new LazyData(source);
        await lazy.Get();
        await lazy.Get();
        expect(source).toHaveBeenCalledOnce();
    });

    it("shares one in-flight request between concurrent callers", async () => {
        const source = vi.fn(async () => 1);
        const lazy = new LazyData(source);
        await Promise.all([lazy.Get(), lazy.Get(), lazy.Get()]);
        expect(source).toHaveBeenCalledOnce();
    });

    it("reloads once after being marked dirty, then caches again", async () => {
        let n = 0;
        const source = vi.fn(async () => ++n);
        const lazy = new LazyData(source);
        expect(await lazy.Get()).toBe(1);
        lazy.MarkDirty();
        expect(await lazy.Get()).toBe(2);
        expect(await lazy.Get()).toBe(2);
        expect(source).toHaveBeenCalledTimes(2);
    });

    it("does not cache a failure", async () => {
        const source = vi.fn<() => Promise<number>>()
            .mockRejectedValueOnce(new Error("down"))
            .mockResolvedValueOnce(7);
        const lazy = new LazyData(source);
        await expect(lazy.Get()).rejects.toThrow("down");
        expect(await lazy.Get()).toBe(7);
    });
});

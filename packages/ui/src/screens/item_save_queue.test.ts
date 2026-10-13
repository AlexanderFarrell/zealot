import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getItemSaveQueue, ItemSaveQueue } from "./item_save_queue";

let calls: Array<{ url: string; body: Record<string, unknown> }>;
let fail: boolean;

beforeEach(() => {
    calls = [];
    fail = false;
    vi.useFakeTimers();
    vi.stubGlobal("fetch", vi.fn(async (url: string, init?: RequestInit) => {
        if (init?.method === "PATCH") {
            calls.push({ url, body: JSON.parse(String(init.body)) });
            if (fail) return new Response("{}", { status: 500 });
            return new Response(JSON.stringify({ item_id: 1, title: "t", content: "" }), { status: 200 });
        }
        return new Response("{}", { status: 200 });
    }));
});

afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
});

describe("ItemSaveQueue", () => {
    it("coalesces edits made inside the delay into one save", async () => {
        const queue = new ItemSaveQueue(1);
        queue.enqueue({ title: "a" });
        queue.enqueue({ content: "b" });
        expect(queue.dirty).toBe(true);
        await vi.advanceTimersByTimeAsync(700);
        expect(calls).toHaveLength(1);
        expect(calls[0]?.body).toMatchObject({ item_id: 1, title: "a", content: "b" });
        expect(queue.dirty).toBe(false);
    });

    it("flushes immediately and reports status changes", async () => {
        const queue = new ItemSaveQueue(1);
        const seen: string[] = [];
        queue.subscribe((s) => seen.push(s));
        queue.enqueue({ title: "x" });
        expect(await queue.flush()).toBe(true);
        expect(seen).toEqual(["saved", "saving", "saved"]);
    });

    it("keeps the patch and reports failure, then retries with later edits merged", async () => {
        const queue = new ItemSaveQueue(1);
        fail = true;
        queue.enqueue({ title: "keep" });
        expect(await queue.flush()).toBe(false);
        expect(queue.dirty).toBe(true);
        fail = false;
        queue.enqueue({ content: "later" });
        expect(await queue.flush()).toBe(true);
        expect(calls[calls.length - 1]?.body).toMatchObject({ title: "keep", content: "later" });
    });

    it("hands back the same queue per item", () => {
        expect(getItemSaveQueue(42)).toBe(getItemSaveQueue(42));
        expect(getItemSaveQueue(42)).not.toBe(getItemSaveQueue(43));
    });
});

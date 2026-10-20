import { beforeEach, describe, expect, it, vi } from "vitest";

async function load() {
    vi.resetModules();
    return (await import("./mobile_pane")).MobilePaneState;
}

beforeEach(() => {
    localStorage.clear();
    document.body.removeAttribute("data-mobile-pane");
    document.body.removeAttribute("data-mobile-side");
});

describe("MobilePaneState", () => {
    it("starts on content", async () => {
        expect((await load()).getPane()).toBe("content");
    });

    it("remembers Related for the next item, across a reload", async () => {
        const state = await load();
        state.setPane("side");
        const reloaded = await load();
        expect(reloaded.getPane()).toBe("side");
        reloaded.setPane("content");
        expect((await load()).getPane()).toBe("content");
    });

    it("keeps the choice when a screen has no side view and when the next one does", async () => {
        const state = await load();
        state.setPane("side");
        state.setSide({ label: "Related", count: null });
        state.setSide(null);
        expect(document.body.hasAttribute("data-mobile-side")).toBe(false);
        state.setSide({ label: "Related", count: null });
        expect(state.getPane()).toBe("side");
        expect(document.body.getAttribute("data-mobile-pane")).toBe("side");
        expect(document.body.hasAttribute("data-mobile-side")).toBe(true);
    });

    it("updates the count only while a side view exists", async () => {
        const state = await load();
        state.setSideCount(3);
        expect(state.getSide()).toBeNull();
        state.setSide({ label: "Related", count: null });
        state.setSideCount(8);
        expect(state.getSide()).toEqual({ label: "Related", count: 8 });
    });

    it("notifies subscribers until unsubscribed", async () => {
        const state = await load();
        const fn = vi.fn();
        const off = state.subscribe(fn);
        state.setPane("side");
        off();
        state.setPane("content");
        expect(fn).toHaveBeenCalledTimes(1);
    });

    it("survives unavailable storage", async () => {
        const state = await load();
        const spy = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked"); });
        expect(() => state.setPane("side")).not.toThrow();
        expect(state.getPane()).toBe("side");
        spy.mockRestore();
    });
});

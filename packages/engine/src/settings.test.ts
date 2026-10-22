import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

async function loadSettings() {
    vi.resetModules();
    return (await import("./settings")).AppSettings;
}

beforeEach(() => {
    localStorage.clear();
    vi.useFakeTimers();
});
afterEach(() => vi.useRealTimers());

const defaults = { item_actions: "toolbar", interface_size: "default", start_screen: "home" };

describe("AppSettings", () => {
    it("starts from defaults", async () => {
        const settings = await loadSettings();
        expect(settings.get()).toEqual({ ...defaults, theme: "light", show_tips: true });
    });

    it("restores saved values over defaults and survives corrupt storage", async () => {
        localStorage.setItem("zealot_settings", JSON.stringify({ theme: "dark" }));
        expect((await loadSettings()).get()).toEqual({ ...defaults, theme: "dark", show_tips: true });
        localStorage.setItem("zealot_settings", "{not json");
        expect((await loadSettings()).get().theme).toBe("light");
    });

    it("persists changes and notifies until unsubscribed", async () => {
        const settings = await loadSettings();
        const listener = vi.fn();
        const unsubscribe = settings.subscribe(listener);
        settings.set({ theme: "dark" });
        expect(JSON.parse(localStorage.getItem("zealot_settings")!)).toMatchObject({ theme: "dark" });
        unsubscribe();
        settings.set({ show_tips: false });
        expect(listener).toHaveBeenCalledOnce();
    });

    it("debounces server sync to the last value", async () => {
        const settings = await loadSettings();
        const sync = vi.fn();
        settings.registerSync(sync);
        settings.set({ theme: "dark" });
        settings.set({ show_tips: false });
        expect(sync).not.toHaveBeenCalled();
        await vi.advanceTimersByTimeAsync(500);
        expect(sync).toHaveBeenCalledOnce();
        expect(sync).toHaveBeenCalledWith({ ...defaults, theme: "dark", show_tips: false });
    });

    it("lets server settings win without syncing back", async () => {
        const settings = await loadSettings();
        const sync = vi.fn();
        settings.registerSync(sync);
        settings.loadFromServer({ theme: "dark" });
        await vi.advanceTimersByTimeAsync(1000);
        expect(settings.get().theme).toBe("dark");
        expect(sync).not.toHaveBeenCalled();
    });
});

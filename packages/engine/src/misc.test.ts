import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getDesktopMode, getRemoteHost, setDesktopMode } from "./desktop_mode";
import { logError, logInfo } from "./log";
import { Store } from "./store/store";
import { clear_hotkeys, Hotkey, register_hotkey, ALT_KEY, CTRL_OR_META_KEY, SHIFT_KEY } from "./ui/hotkeys";

describe("Store", () => {
    it("merges partial state and notifies subscribers", () => {
        const store = new Store({ a: 1, b: "x" });
        const seen: Array<{ a: number; b: string }> = [];
        store.subscribe((s) => seen.push(s));
        store.set({ a: 2 });
        expect(store.get()).toEqual({ a: 2, b: "x" });
        expect(seen).toEqual([{ a: 2, b: "x" }]);
    });
});

describe("desktop mode", () => {
    it("is null on the web, remembers the host only in remote mode", () => {
        expect(getDesktopMode()).toBeNull();
        setDesktopMode("remote", "zealot.example");
        expect(getDesktopMode()).toBe("remote");
        expect(getRemoteHost()).toBe("zealot.example");
        setDesktopMode("local", "ignored");
        expect(getRemoteHost()).toBeNull();
    });
});

describe("logging", () => {
    it("prefixes messages and appends context as json", () => {
        const info = vi.spyOn(console, "info").mockImplementation(() => {});
        const error = vi.spyOn(console, "error").mockImplementation(() => {});
        logInfo("hello");
        logError("bad", { status: 500 });
        expect(info).toHaveBeenCalledWith("[zealot] hello");
        expect(error).toHaveBeenCalledWith('[zealot] bad {"status":500}');
        vi.restoreAllMocks();
    });
});

describe("Hotkey", () => {
    it("describes itself with modifiers in a fixed order", () => {
        expect(new Hotkey("k", [SHIFT_KEY, CTRL_OR_META_KEY, ALT_KEY]).toString()).toBe("CTRL+ALT+SHIFT+K");
        expect(new Hotkey("Escape", []).toString()).toBe("ESCAPE");
    });

    describe("dispatch", () => {
        const run = vi.fn();
        beforeEach(() => {
            clear_hotkeys();
            run.mockClear();
            register_hotkey(new Hotkey("k", [CTRL_OR_META_KEY], run));
        });
        afterEach(() => clear_hotkeys());

        function press(init: KeyboardEventInit, target: EventTarget = document) {
            const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
            target.dispatchEvent(event);
            return event;
        }

        it("runs the handler for a matching chord and prevents default", () => {
            const mac = navigator.platform.toUpperCase().includes("MAC");
            const event = press({ code: "KeyK", key: "k", ctrlKey: !mac, metaKey: mac });
            expect(run).toHaveBeenCalledOnce();
            expect(event.defaultPrevented).toBe(true);
        });

        it("ignores the key without its modifier", () => {
            press({ code: "KeyK", key: "k" });
            expect(run).not.toHaveBeenCalled();
        });

        it("ignores keys typed into inputs", () => {
            const input = document.createElement("input");
            document.body.appendChild(input);
            const mac = navigator.platform.toUpperCase().includes("MAC");
            press({ code: "KeyK", key: "k", ctrlKey: !mac, metaKey: mac }, input);
            expect(run).not.toHaveBeenCalled();
            input.remove();
        });
    });
});

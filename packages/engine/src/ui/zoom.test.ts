import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import runner from "./commands";
import { clear_hotkeys, Hotkey, register_hotkey, CTRL_OR_META_KEY } from "./hotkeys";
import {
    ZOOM_DEFAULT, ZOOM_KEY, ZOOM_STEPS, canZoomIn, canZoomOut, getZoom, initZoom, loadZoom,
    registerZoomCommands, resetZoom, setZoom, snapZoom, stepZoom, zoomIn, zoomOut,
} from "./zoom";

describe("zoom steps", () => {
    it("covers 25 to 500 percent", () => {
        expect(ZOOM_STEPS[0]).toBe(25);
        expect(ZOOM_STEPS[ZOOM_STEPS.length - 1]).toBe(500);
    });

    it("steps one at a time and stops at the ends", () => {
        expect(stepZoom(100, 1)).toBe(110);
        expect(stepZoom(100, -1)).toBe(90);
        expect(stepZoom(500, 1)).toBe(500);
        expect(stepZoom(25, -1)).toBe(25);
    });

    it("snaps between steps to the nearest and defaults bad values to 100", () => {
        expect(snapZoom(118)).toBe(125);
        expect(snapZoom("66")).toBe(67);
        expect(snapZoom(9999)).toBe(500);
        expect(snapZoom(1)).toBe(25);
        for (const bad of [null, undefined, "", "abc", NaN, -5, 0]) expect(snapZoom(bad)).toBe(ZOOM_DEFAULT);
    });
});

describe("zoom state", () => {
    const apply = vi.fn();
    beforeEach(() => {
        localStorage.clear();
        apply.mockClear();
        initZoom(apply);
    });

    it("starts at 100 and applies the stored level at startup", () => {
        expect(getZoom()).toBe(100);
        localStorage.setItem(ZOOM_KEY, "150");
        initZoom(apply);
        expect(getZoom()).toBe(150);
        expect(apply).toHaveBeenLastCalledWith(150);
    });

    it("persists changes and falls back to 100 on junk", () => {
        zoomIn();
        expect(localStorage.getItem(ZOOM_KEY)).toBe("110");
        expect(loadZoom()).toBe(110);
        localStorage.setItem(ZOOM_KEY, "junk");
        expect(loadZoom()).toBe(100);
    });

    it("reports when it can't go further", () => {
        setZoom(500);
        expect(canZoomIn()).toBe(false);
        setZoom(25);
        expect(canZoomOut()).toBe(false);
        zoomOut();
        expect(getZoom()).toBe(25);
        resetZoom();
        expect(getZoom()).toBe(100);
    });
});

describe("zoom hotkeys", () => {
    function press(init: KeyboardEventInit, target: EventTarget = document) {
        const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
        target.dispatchEvent(event);
        return event;
    }
    beforeEach(() => {
        localStorage.clear();
        runner.clear();
        initZoom(vi.fn());
        registerZoomCommands();
    });
    afterEach(() => runner.clear());

    it("zooms with = + - and 0, by physical key, including numpad", () => {
        press({ key: "=", code: "Equal", ctrlKey: true });
        expect(getZoom()).toBe(110);
        press({ key: "+", code: "Equal", ctrlKey: true, shiftKey: true });
        expect(getZoom()).toBe(125);
        press({ key: "+", code: "NumpadAdd", ctrlKey: true });
        expect(getZoom()).toBe(150);
        press({ key: "-", code: "Minus", ctrlKey: true });
        press({ key: "-", code: "NumpadSubtract", ctrlKey: true });
        expect(getZoom()).toBe(110);
        press({ key: "0", code: "Digit0", ctrlKey: true });
        expect(getZoom()).toBe(100);
    });

    it("works while typing in a text field", () => {
        const input = document.createElement("input");
        document.body.appendChild(input);
        press({ key: "=", code: "Equal", ctrlKey: true }, input);
        expect(getZoom()).toBe(110);
        input.remove();
    });

    it("leaves other hotkeys blocked while typing", () => {
        const other = vi.fn();
        register_hotkey(new Hotkey("k", [CTRL_OR_META_KEY], other));
        const input = document.createElement("input");
        document.body.appendChild(input);
        press({ key: "k", code: "KeyK", ctrlKey: true }, input);
        expect(other).not.toHaveBeenCalled();
        press({ key: "k", code: "KeyK", ctrlKey: true });
        expect(other).toHaveBeenCalledOnce();
        input.remove();
        clear_hotkeys();
    });

    it("lists the three commands with their hotkeys for Ctrl+P", () => {
        const names = ["Zoom In", "Zoom Out", "Reset Zoom"];
        for (const n of names) expect(runner.Commands.get(n)!.hotkeys.length).toBeGreaterThan(0);
        expect(runner.Commands.get("Zoom In")!.hotkeys[0]!.toString()).toBe("CTRL+=");
    });
});

import { beforeEach, describe, expect, it } from "vitest";
import { MobilePaneState } from "@websoil/engine";
import "./mobile_pane_switch";

beforeEach(() => {
    localStorage.clear();
    document.body.innerHTML = "";
    MobilePaneState.setSide(null);
    MobilePaneState.setPane("content");
});

function mount(): HTMLElement {
    const strip = document.createElement("mobile-pane-switch");
    document.body.appendChild(strip);
    return strip;
}

function tab(strip: HTMLElement, index: number): HTMLButtonElement {
    const button = strip.querySelectorAll("button")[index];
    if (!button) throw new Error(`No tab ${index}`);
    return button;
}

describe("mobile-pane-switch", () => {
    it("is hidden when the screen has no side view", () => {
        expect(mount().hidden).toBe(true);
    });

    it("shows Content and the labelled count, and switches panes on tap", () => {
        const strip = mount();
        MobilePaneState.setSide({ label: "Related", count: null });
        MobilePaneState.setSideCount(8);
        const content = tab(strip, 0);
        const related = tab(strip, 1);
        expect(strip.hidden).toBe(false);
        expect(content.textContent).toBe("Content");
        expect(related.textContent).toBe("Related (8)");
        expect(content.getAttribute("aria-selected")).toBe("true");

        related.click();
        expect(MobilePaneState.getPane()).toBe("side");
        expect(tab(strip, 1).getAttribute("aria-selected")).toBe("true");

        tab(strip, 0).click();
        expect(MobilePaneState.getPane()).toBe("content");
    });

    it("stays on Related when the next item loads its side view", () => {
        const strip = mount();
        MobilePaneState.setSide({ label: "Related", count: 2 });
        tab(strip, 1).click();
        MobilePaneState.setSide(null);
        MobilePaneState.setSide({ label: "Related", count: 5 });
        expect(tab(strip, 1).getAttribute("aria-selected")).toBe("true");
        expect(tab(strip, 1).textContent).toBe("Related (5)");
    });
});

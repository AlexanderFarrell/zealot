import { describe, expect, it } from "vitest";
import { createItemTitleElement, renderItemTitle } from "./item_title";

describe("createItemTitleElement", () => {
    it("renders only the text when there is no icon", () => {
        const element = createItemTitleElement({ Title: "Plain", RawIcon: "" });
        expect(element.className).toBe("item-title-inline");
        expect(element.children).toHaveLength(1);
        expect(element.textContent).toBe("Plain");
    });

    it("renders an emoji icon literally and a known shortcode as its emoji", () => {
        const literal = createItemTitleElement({ Title: "T", RawIcon: "🧪" });
        expect(literal.querySelector(".item-title-inline-icon--text")?.textContent).toBe("🧪");
        const shortcode = createItemTitleElement({ Title: "T", RawIcon: ":smile:" });
        expect(shortcode.querySelector(".item-title-inline-icon--text")?.textContent).not.toBe(":smile:");
    });

    it("keeps an unknown shortcode as plain text", () => {
        const element = createItemTitleElement({ Title: "T", RawIcon: ":no_such_emoji_zz:" });
        expect(element.querySelector(".item-title-inline-icon--text")?.textContent).toBe(":no_such_emoji_zz:");
    });

    it("never interprets the title as html", () => {
        const element = createItemTitleElement({ Title: "<img src=x onerror=alert(1)>", RawIcon: "" });
        expect(element.querySelector("img")).toBeNull();
        expect(element.textContent).toBe("<img src=x onerror=alert(1)>");
    });

    it("applies the extra class names", () => {
        const element = createItemTitleElement({ Title: "T", RawIcon: "x" }, { className: "a", iconClassName: "b", textClassName: "c" });
        expect(element.classList.contains("a")).toBe(true);
        expect(element.querySelector(".b")).not.toBeNull();
        expect(element.querySelector(".item-title-text.c")).not.toBeNull();
    });
});

describe("renderItemTitle", () => {
    it("replaces the container children", () => {
        const container = document.createElement("div");
        container.innerHTML = "<p>old</p><p>older</p>";
        renderItemTitle(container, { Title: "New", RawIcon: "" });
        expect(container.children).toHaveLength(1);
        expect(container.textContent).toBe("New");
    });
});

import { describe, expect, it } from "vitest";
import content, { icons, Stylesheet } from "./index";

describe("content", () => {
    it("exposes icons and the stylesheet on the default export", () => {
        expect(content.icons).toBe(icons);
        expect(content.Stylesheet).toBe(Stylesheet);
    });

    it("gives every icon a non-empty asset reference", () => {
        const entries = Object.entries(icons);
        expect(entries.length).toBeGreaterThan(50);
        for (const [name, value] of entries) {
            expect(typeof value, name).toBe("string");
            expect(value, name).not.toBe("");
        }
    });

    it("keeps icon names in camelCase so lookups by key stay predictable", () => {
        for (const name of Object.keys(icons)) {
            expect(name).toMatch(/^[a-z][A-Za-z0-9]*$/);
        }
    });
});

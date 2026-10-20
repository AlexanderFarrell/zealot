import { describe, expect, it } from "vitest";
import { formatTimestamp } from "./item_timestamps";

describe("formatTimestamp", () => {
    it("formats Unix seconds as a local YYYY-MM-DD date", () => {
        const noon = new Date(2026, 0, 1, 12, 0, 0).getTime() / 1000;
        expect(formatTimestamp(noon)).toBe("2026-01-01");
    });

    it("is empty when the timestamp is unknown", () => {
        expect(formatTimestamp(0)).toBe("");
        expect(formatTimestamp(Number.NaN)).toBe("");
    });
});

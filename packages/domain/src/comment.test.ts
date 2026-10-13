import { DateTime } from "luxon";
import { describe, expect, it } from "vitest";
import { Comment, formatCommentTimestamp, parseCommentTimestamp } from "./comment";

describe("parseCommentTimestamp", () => {
    it("parses the SQL form the server returns", () => {
        const parsed = parseCommentTimestamp("2026-10-13 09:05:07");
        expect(parsed.isValid).toBe(true);
        expect(parsed.toFormat("yyyy-MM-dd HH:mm:ss")).toBe("2026-10-13 09:05:07");
    });

    it("falls back to ISO", () => {
        const parsed = parseCommentTimestamp("2026-10-13T09:05:07Z");
        expect(parsed.isValid).toBe(true);
        expect(parsed.toUTC().hour).toBe(9);
    });

    it("returns an invalid DateTime that names the bad input", () => {
        const parsed = parseCommentTimestamp("not a date");
        expect(parsed.isValid).toBe(false);
        expect(parsed.invalidReason).toContain("not a date");
    });
});

describe("formatCommentTimestamp", () => {
    it("round-trips through parse", () => {
        const text = "2026-01-02 03:04:05";
        expect(formatCommentTimestamp(parseCommentTimestamp(text))).toBe(text);
        expect(formatCommentTimestamp(DateTime.fromObject({ year: 2026, month: 3, day: 4, hour: 5, minute: 6, second: 7 }))).toBe("2026-03-04 05:06:07");
    });
});

describe("Comment", () => {
    it("maps dto fields and parses the timestamp", () => {
        const comment = new Comment({
            comment_id: 4,
            item: { item_id: 1, title: "T", content: "" },
            timestamp: "2026-10-13 09:05:07",
            content: "hello",
        });
        expect(comment.CommentID).toBe(4);
        expect(comment.Item.ItemID).toBe(1);
        expect(comment.Content).toBe("hello");
        expect(comment.Timestamp.year).toBe(2026);
    });
});

import { describe, expect, it } from "vitest";
import { escapeTableCell, isMarkdownTableSeparator, splitTableRow } from "./table_utils";

describe("splitTableRow", () => {
    it("returns no cells for a blank line", () => {
        expect(splitTableRow("   ")).toEqual([]);
    });

    it("splits with or without outer pipes and trims cells", () => {
        expect(splitTableRow("| a | b |")).toEqual(["a", "b"]);
        expect(splitTableRow("a|b|c")).toEqual(["a", "b", "c"]);
    });

    it("keeps escaped pipes inside a cell", () => {
        expect(splitTableRow("| a \\| b | c |")).toEqual(["a | b", "c"]);
    });

    it("keeps a trailing lone backslash", () => {
        expect(splitTableRow("a|b\\")).toEqual(["a", "b\\"]);
    });

    it("keeps empty cells", () => {
        expect(splitTableRow("| | x |")).toEqual(["", "x"]);
    });
});

describe("isMarkdownTableSeparator", () => {
    it("accepts alignment markers", () => {
        expect(isMarkdownTableSeparator("| --- | :---: | ---: |")).toBe(true);
    });

    it("accepts short dashes; rejects empty cells, text and a wrong column count", () => {
        expect(isMarkdownTableSeparator("| -- | --- |")).toBe(true); // GitHub-style aligned `--:` cells
        expect(isMarkdownTableSeparator("| | --- |")).toBe(false);
        expect(isMarkdownTableSeparator("| --- | abc |")).toBe(false);
        expect(isMarkdownTableSeparator("| --- | --- |", 3)).toBe(false);
        expect(isMarkdownTableSeparator("")).toBe(false);
    });
});

describe("escapeTableCell", () => {
    it("escapes backslashes before pipes so the result round-trips", () => {
        const text = "a|b\\c";
        expect(escapeTableCell(text)).toBe("a\\|b\\\\c");
        expect(splitTableRow(`| ${escapeTableCell(text)} |`)).toEqual(["a|b\\c"]);
    });
});

import { describe, expect, it } from "vitest";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";

const roundTrip = (md: string): string => serializeZealotScript(parseZealotScript(ZealotSchema, md));

// Every construct ZealotScript stores must survive load -> save unchanged, otherwise
// merely opening a document rewrites it.
const STABLE: Array<[string, string]> = [
	["heading", "## Heading with **bold**"],
	["paragraphs", "one\n\ntwo"],
	["inline marks", "**b** *i* ~~s~~ _u_ `code`"],
	["nested marks", "**bold *and italic***"],
	["html marks", "<mark>hl</mark> <sub>2</sub> <sup>3</sup>"],
	["color", "<color:red>warm</color> text"],
	["hard break", "line<br>next"],
	["wikilink", "see [[Z233]] and [[Z233|the ticket]]"],
	["wikilink anchor", "[[Z233#Spec|spec]]"],
	["type link", "[[type:Ticket]]"],
	["markdown link", "[site](https://example.com/a?b=c)"],
	["image", "![alt](/api/media/x.png)"],
	["date refs", "due @2026-10-13 in @2026-W42 of @2026"],
	["inline math", "area is $a^2$ here"],
	["math block", ":::math\nE = mc^2\n:::"],
	["youtube", ":::youtube dQw4w9WgXcQ"],
	["code block", "```ts\nconst a = 1;\n\nconst b = 2;\n```"],
	["view embed", "![[AI Queue]]"],
	["view embed table", "![[AI Queue|table]]"],
	["view embed in quote", "> ![[AI Queue|cards]]"],
	["view embed in admonition", ":::note\n![[AI Queue]]\n:::"],
	["view embed in details", ":::details More\n![[AI Queue|table]]\n:::"],
	["view embed in column", ":::columns\n:::col\n![[AI Queue]]\n:::\n:::col\nright\n:::\n:::"],
	["view embed in tab", ":::tabs\n:::tab One\n![[AI Queue]]\n:::\n:::tab Two\nsecond\n:::\n:::"],
	["inline embed text stays text", "see ![[AI Queue]] here"],
	["unknown suffix stays text", "![[AI Queue|label]]"],
	["longer fence", "````zealot-view\n{}\n````"],
	["tilde fence", "~~~zealot-view\n{}\n~~~"],
	["fence holding a fence", "````md\n```ts\nx\n```\n````"],
	["blockquote", "> quoted\n>\n> - item"],
	["hr", "text\n\n---\n\ntext"],
	["bullet nesting", "- a\n\t- b\n- c"],
	["ordered", "1. a\n2. b"],
	["task list", "- [ ] a\n- [x] b"],
	["table", "| a | b |\n| --- | --- |\n| 1 | 2 |"],
	["table escaped pipe", "| a | b |\n| --- | --- |\n| x \\| y | 2 |"],
	["headerless table", ":::table\na | b\nc | d\n:::"],
	["admonition", ":::warning\ncareful\n:::"],
	["details", ":::details More\nhidden\n:::"],
	["spoiler", ":::spoiler\nsecret\n:::"],
	["definition list", ":::definition\nterm\n: meaning\n:::"],
	["columns", ":::columns\n:::col\nleft\n:::\n:::col\nright\n:::\n:::"],
	["tabs", ":::tabs\n:::tab One\nfirst\n:::\n:::tab Two\nsecond\n:::\n:::"],
	["partial italic inside bold", "**bold *and italic* more**"],
	["plain star and path", "2 \\* 3 \\* 4 in C:\\\\dir"],
	["bold run then plain", "**a** b **c**"],
	["escaped marks", "a \\*literal\\* star"],
];

describe("round trip", () => {
	it.each(STABLE)("%s", (_name, md) => {
		expect(roundTrip(md)).toBe(md);
	});

	it("is idempotent after one pass for hard-wrapped text", () => {
		const once = roundTrip("a\nb");
		expect(roundTrip(once)).toBe(once);
	});

	it("keeps an empty line between paragraphs stable", () => {
		const md = "one\n\n\ntwo";
		expect(roundTrip(roundTrip(md))).toBe(roundTrip(md));
	});

	it("serialises link + marks together", () => {
		expect(roundTrip("[**bold link**](https://example.com)")).toBe("[**bold link**](https://example.com)");
	});

	it("converts an emoji shortcode to the character on load", () => {
		expect(roundTrip("ship it :rocket: now")).toBe("ship it 🚀 now");
	});

	it("pads short table rows to the header width", () => {
		expect(roundTrip("| a | b |\n| --- | --- |\n| 1 |")).toBe("| a | b |\n| --- | --- |\n| 1 |  |");
	});

	it("keeps a table together across a blank line and ends it at prose", () => {
		const out = roundTrip("| a |\n| --- |\n| 1 |\n\n| 2 |\n\nafter");
		expect(out).toBe("| a |\n| --- |\n| 1 |\n| 2 |\n\nafter");
	});

	it("reads a header row inside a :::table block", () => {
		expect(roundTrip(":::table\na | b\n--- | ---\n1 | 2\n:::")).toBe("| a | b |\n| --- | --- |\n| 1 | 2 |");
	});

	it("parses admonition bodies with headings, lists, code and tables", () => {
		const md = ":::tip\n# Title\n\n- a\n- b\n\n```\ncode\n```\n:::";
		expect(roundTrip(md)).toBe(md);
	});

	it("serialises an unknown :::kind as a note", () => {
		expect(roundTrip(":::bogus\nx\n:::")).toBe(":::note\nx\n:::");
	});
});

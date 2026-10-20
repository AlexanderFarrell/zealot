import { describe, expect, it } from "vitest";
import { EditorState, TextSelection } from "prosemirror-state";
import { baseKeymap } from "prosemirror-commands";
import { splitListItem } from "prosemirror-schema-list";
import { chainCommands } from "prosemirror-commands";
import type { Node as PMNode } from "prosemirror-model";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { importMarkdown, looksLikeMarkdown } from "./import_markdown";
import { autolinkPlugin } from "./autolink_plugin";
import { ensureTrailingParagraph, stripTrailingParagraph, trailingParagraphPlugin } from "./trailing_paragraph";
import { exitCodeBlockOnBlankLine } from "./commands";
import { hrefForUrl, isUrlStart, matchBareUrl } from "./linkify";
import { taskItemInputRule, viewEmbedInputRule, wikilinkInputRule, WIKILINK_TYPED_RE, wikilinkHref } from "./link_input_rules";

const parse = (md: string): PMNode => parseZealotScript(ZealotSchema, md);
const roundTrip = (md: string): string => serializeZealotScript(parse(md));
const imported = (md: string): string => roundTrip(importMarkdown(md));

const endOf = (doc: PMNode): number => doc.content.size;
const stateAt = (doc: PMNode, pos: number, plugins = [autolinkPlugin()]): EditorState =>
	EditorState.create({ doc, plugins, selection: TextSelection.near(doc.resolve(pos)) });

describe("glitch 1: leaving tables, lists, code and quotes", () => {
	it("an empty bullet item survives a save/load cycle instead of becoming a stray '-'", () => {
		const md = "- a\n- \n- c";
		const doc = parse(md);
		expect(doc.childCount).toBe(1);
		expect(doc.child(0).type.name).toBe("bullet_list");
		expect(doc.child(0).childCount).toBe(3);
		expect(roundTrip(md)).toBe("- a\n-\n- c");
	});

	it("a bare marker is an empty ordered item too", () => {
		expect(parse("1. a\n2.").child(0).childCount).toBe(2);
	});

	it("Enter in an empty list item leaves the list", () => {
		const doc = parse("- a\n- ");
		// Cursor inside the empty second item.
		let pos = 0;
		doc.descendants((node, p) => { if (node.type.name === "paragraph" && node.content.size === 0) pos = p + 1; });
		const state = stateAt(doc, pos, []);
		let next = state;
		const enter = chainCommands(splitListItem(ZealotSchema.nodes["list_item"]!), baseKeymap["Enter"]!);
		const ok = enter(state, (tr) => { next = state.apply(tr); });
		expect(ok).toBe(true);
		expect(next.doc.lastChild?.type.name).toBe("paragraph");
		expect(next.doc.child(0).childCount).toBe(1);
	});

	it("a document ending in a table gets an editing paragraph that is stripped on save", () => {
		const md = "| a | b |\n| --- | --- |\n| 1 | 2 |";
		const doc = parse(md);
		expect(doc.lastChild?.type.name).toBe("table");
		const editing = ensureTrailingParagraph(doc);
		expect(editing.lastChild?.type.name).toBe("paragraph");
		expect(serializeZealotScript(stripTrailingParagraph(editing))).toBe(md);
	});

	it.each([
		["code block", "```ts\nconst a = 1;\n```"],
		["blockquote", "> quoted"],
		["bullet list", "- a\n- b"],
		["admonition", ":::note\ntext\n:::"],
	])("a document ending in a %s gets a trailing paragraph", (_name, md) => {
		expect(ensureTrailingParagraph(parse(md)).lastChild?.type.name).toBe("paragraph");
	});

	it("the trailing paragraph plugin restores the paragraph after the user deletes it", () => {
		const doc = ensureTrailingParagraph(parse("- a"));
		const state = EditorState.create({ doc, plugins: [trailingParagraphPlugin()] });
		const tr = state.tr.delete(endOf(doc) - 2, endOf(doc));
		const next = state.apply(tr);
		expect(next.doc.lastChild?.type.name).toBe("paragraph");
	});

	it("a user-typed empty paragraph between blocks is not stripped", () => {
		const doc = parse("> q\n\ntext");
		expect(stripTrailingParagraph(doc).childCount).toBe(doc.childCount);
	});

	it("Enter on an empty last line of a code block exits to a new paragraph", () => {
		const doc = parse("```\nline\n```");
		const codeEnd = doc.child(0).content.size;
		const withNewline = doc.child(0).type.create(null, ZealotSchema.text("line\n"));
		const next = ZealotSchema.nodes["doc"]!.create(null, [withNewline]);
		const state = stateAt(next, 1 + codeEnd + 1, []);
		let after = state;
		expect(exitCodeBlockOnBlankLine(state, (tr) => { after = state.apply(tr); })).toBe(true);
		expect(after.doc.child(0).textContent).toBe("line");
		expect(after.doc.lastChild?.type.name).toBe("paragraph");
		expect(after.selection.$from.parent.type.name).toBe("paragraph");
	});

	it("Enter mid code block is left to the default handler", () => {
		const doc = parse("```\nline\n```");
		expect(exitCodeBlockOnBlankLine(stateAt(doc, 3, []))).toBe(false);
		expect(baseKeymap["Enter"]).toBeDefined();
	});
});

describe("glitch 2: bare URLs become links", () => {
	it("parses a bare https URL into a link mark", () => {
		const para = parse("see https://example.com/x now").child(0);
		const link = para.child(1);
		expect(link.text).toBe("https://example.com/x");
		expect(link.marks[0]?.type.name).toBe("link");
		expect(link.marks[0]?.attrs.href).toBe("https://example.com/x");
	});

	it("round-trips a bare URL without turning it into [url](url)", () => {
		expect(roundTrip("see https://example.com/x now")).toBe("see https://example.com/x now");
	});

	it("excludes sentence punctuation and an unbalanced bracket", () => {
		expect(matchBareUrl("https://a.b/c.")).toBe("https://a.b/c");
		expect(matchBareUrl("https://a.b/c),")).toBe("https://a.b/c");
		expect(matchBareUrl("https://en.wikipedia.org/wiki/Foo_(bar)")).toBe("https://en.wikipedia.org/wiki/Foo_(bar)");
	});

	it("does not treat intraword text as a URL start", () => {
		expect(isUrlStart("xhttps://a.b", 1)).toBe(false);
		expect(isUrlStart("a:https://a.b", 2)).toBe(true);
	});

	it("gives www. URLs a scheme", () => {
		expect(hrefForUrl("www.example.com")).toBe("https://www.example.com");
	});

	it("parses <url> autolinks and drops link titles", () => {
		expect(roundTrip("<https://example.com>")).toBe("https://example.com");
		expect(roundTrip('[x](https://e.com "title")')).toBe("[x](https://e.com)");
	});

	it("keeps parenthesised URLs inside a link", () => {
		const md = "[a](https://en.wikipedia.org/wiki/Foo_(bar)) after";
		expect(roundTrip(md)).toBe(md);
	});

	it("links a URL when whitespace is typed after it", () => {
		const doc = parse("go https://example.com ");
		const state = stateAt(doc, endOf(doc) - 1);
		const tr = state.tr.insertText("x", endOf(doc) - 1); // any docChanged tr
		const next = state.apply(tr);
		const linked = next.doc.child(0).child(1);
		expect(linked.marks.some((m) => m.type.name === "link")).toBe(true);
	});

	it("does not link a URL while it is still being typed at the cursor", () => {
		const base = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("go ")),
		]);
		const state = stateAt(base, endOf(base) - 1);
		const next = state.apply(state.tr.insertText("https://exam", endOf(base) - 1));
		expect(next.doc.textContent).toBe("go https://exam");
		expect(next.doc.firstChild?.lastChild?.marks).toHaveLength(0);
	});
});

describe("glitch 3: links at the end of bullet points", () => {
	it("links a URL jammed against its label at the end of an item", () => {
		const para = parse("- docs:https://example.com").child(0).child(0).child(0);
		expect(para.child(0).text).toBe("docs:");
		expect(para.child(1).marks[0]?.attrs.href).toBe("https://example.com");
	});

	it("links the URL at the end of an item when Enter splits the item", () => {
		const base = parse("- see https://example.com");
		const state = stateAt(base, 3, [autolinkPlugin()]);
		// Place cursor at end of item text, then split the list item.
		let pos = 0;
		base.descendants((node, p) => { if (node.isTextblock) pos = p + 1 + node.content.size; });
		const atEnd = state.apply(state.tr.setSelection(TextSelection.create(base, pos)));
		let next = atEnd;
		splitListItem(ZealotSchema.nodes["list_item"]!)(atEnd, (tr) => { next = atEnd.apply(tr); });
		const list = next.doc.child(0);
		expect(list.childCount).toBe(2);
		const firstPara = list.child(0).child(0);
		expect(firstPara.lastChild?.marks[0]?.type.name).toBe("link");
	});

	it("converts a typed [label](url) once whitespace follows it", () => {
		const base = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("see [docs](https://example.com)")),
		]);
		const state = stateAt(base, endOf(base) - 1);
		const next = state.apply(state.tr.insertText(" ", endOf(base) - 1));
		const para = next.doc.child(0);
		expect(para.textContent).toBe("see docs ");
		expect(para.child(1).text).toBe("docs");
		expect(para.child(1).marks[0]?.attrs.href).toBe("https://example.com");
	});

	it("serialises nested items of mixed depth with tabs and reparses them", () => {
		const md = "- a\n\t- b\n\t\t- c";
		expect(roundTrip(md)).toBe(md);
	});
});

describe("glitch 4: imported Markdown", () => {
	it("reads + bullets and 1) ordered items", () => {
		expect(roundTrip("+ one\n+ two")).toBe("- one\n- two");
		expect(roundTrip("1) one\n2) two")).toBe("1. one\n2. two");
	});

	it("nests items indented by 2 spaces", () => {
		expect(roundTrip("- a\n  - b\n    - c")).toBe("- a\n\t- b\n\t\t- c");
	});

	it("keeps task list checkboxes through a round trip", () => {
		const md = "- [ ] todo\n- [x] done";
		expect(roundTrip(md)).toBe(md);
		expect(parse(md).child(0).child(1).attrs.checked).toBe(true);
	});

	it("folds list continuation lines into the item", () => {
		expect(roundTrip("- a\n  continued\n- b")).toBe("- a continued\n- b");
	});

	it("reads aligned table separators and tables without outer pipes", () => {
		expect(roundTrip("| a | b |\n|:--:|--:|\n| 1 | 2 |")).toBe("| a | b |\n| --- | --- |\n| 1 | 2 |");
		expect(roundTrip("a | b\n--- | ---\n1 | 2")).toBe("| a | b |\n| --- | --- |\n| 1 | 2 |");
	});

	it("reads __bold__ and keeps snake_case intact", () => {
		expect(roundTrip("__bold__ here")).toBe("**bold** here");
		expect(roundTrip("a snake_case_name")).toBe("a snake_case_name");
		expect(parse("a snake_case_name").child(0).childCount).toBe(1);
	});

	it("round-trips underline", () => {
		expect(roundTrip("_under_ line")).toBe("_under_ line");
	});

	it("keeps lists, code and nested quotes inside a quote", () => {
		const doc = parse("> intro\n> - a\n> - b\n> > deep");
		const quote = doc.child(0);
		expect(quote.type.name).toBe("blockquote");
		const names: string[] = [];
		quote.forEach((c) => names.push(c.type.name));
		expect(names).toEqual(["paragraph", "bullet_list", "blockquote"]);
	});

	it("importMarkdown joins loose lists, converts ~~~ fences, setext headings and rules", () => {
		expect(importMarkdown("1. a\n\n2. b")).toBe("1. a\n2. b");
		expect(importMarkdown("~~~js\ncode\n~~~")).toBe("```js\ncode\n```");
		expect(importMarkdown("Title\n=====")).toBe("# Title");
		expect(importMarkdown("Sub\n---")).toBe("## Sub");
		expect(importMarkdown("***")).toBe("---");
		expect(importMarkdown("a &amp; b&nbsp;c")).toBe("a & b c");
	});

	it("importMarkdown leaves fenced code untouched", () => {
		const md = "```\n1. a\n\n2. b\n===\n```";
		expect(importMarkdown(md)).toBe(md);
	});

	it("imports a typical ChatGPT answer", () => {
		const md = [
			"## Steps",
			"",
			"1. **Install** the tool",
			"",
			"2. Run `npm test`",
			"   - Expect green",
			"   - See https://example.com/docs.",
			"",
			"| Name | Value |",
			"|------|:-----:|",
			"| a    | 1     |",
		].join("\n");
		const doc = parse(importMarkdown(md));
		const kinds: string[] = [];
		doc.forEach((c) => kinds.push(c.type.name));
		expect(kinds).toEqual(["heading", "paragraph", "ordered_list", "paragraph", "table"]);
		expect(doc.child(2).childCount).toBe(2);
	});

	it("imports a typical Notion export", () => {
		const out = imported("# Page\n\n- [ ] todo\n- [x] done\n\n> note\n\n---\n\n* * *");
		expect(out).toContain("- [ ] todo");
		expect(out).toContain("- [x] done");
		expect(out).toContain("> note");
	});

	it("looksLikeMarkdown is conservative about prose", () => {
		expect(looksLikeMarkdown("just a sentence, with a * star")).toBe(false);
		expect(looksLikeMarkdown("# Heading")).toBe(true);
		expect(looksLikeMarkdown("- item")).toBe(true);
		expect(looksLikeMarkdown("see https://example.com")).toBe(true);
		expect(looksLikeMarkdown("| a | b |\n| --- | --- |")).toBe(true);
	});
});

describe("typed link and task syntax", () => {
	const apply = (rule: NonNullable<ReturnType<typeof wikilinkInputRule>>, doc: PMNode, pos: number, typed: string, from: number) => {
		const state = stateAt(doc, pos, []);
		const match = (rule as unknown as { match: RegExp }).match.exec(typed);
		expect(match).not.toBeNull();
		const handler = (rule as unknown as { handler: (s: EditorState, m: RegExpExecArray, a: number, b: number) => unknown }).handler;
		const tr = handler(state, match!, from, pos) as import("prosemirror-state").Transaction | null;
		return tr ? state.apply(tr) : null;
	};

	it("does not link [a](url) while the cursor is inside the parentheses", () => {
		const base = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("[a]()")),
		]);
		// Cursor between the parentheses, then type the first URL character.
		const state = stateAt(base, 5);
		const next = state.apply(state.tr.insertText("h", 5));
		expect(next.doc.textContent).toBe("[a](h)");
		const typedAll = next.apply(next.tr.insertText("ttps://example.com", 6));
		expect(typedAll.doc.textContent).toBe("[a](https://example.com)");
		expect(typedAll.doc.firstChild?.childCount).toBe(1);
	});

	it("links [a](url) once the cursor has moved past it and text is edited", () => {
		const base = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("[a](https://e.com) x")),
		]);
		const state = stateAt(base, endOf(base) - 1);
		const next = state.apply(state.tr.insertText("y", endOf(base) - 1));
		expect(next.doc.firstChild?.child(0).text).toBe("a");
		expect(next.doc.firstChild?.child(0).marks[0]?.attrs.href).toBe("https://e.com");
	});

	it("typing [[Title]] produces a wiki link mark", () => {
		const rule = wikilinkInputRule(ZealotSchema)!;
		const doc = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("see [[Home]")),
		]);
		// The rule runs with the final "]" about to be inserted: simulate via the pending text.
		const full = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("see [[Home]]")),
		]);
		expect(doc.textContent).toBe("see [[Home]");
		const next = apply(rule, full, 1 + "see [[Home]]".length, "see [[Home]]", 5);
		const link = next!.doc.firstChild!.child(1);
		expect(link.text).toBe("Home");
		expect(link.marks[0]?.attrs.href).toBe("zealot://item/Home");
		expect(serializeZealotScript(next!.doc)).toBe("see [[Home]]");
	});

	it("handles [[Title|label]] and [[Title#Anchor]]", () => {
		expect(WIKILINK_TYPED_RE.exec("[[Z233|the ticket]]")?.[3]).toBe("the ticket");
		expect(wikilinkHref("Z233", "Spec")).toBe("zealot://item/Z233#Spec");
		expect(wikilinkHref("type:Ticket")).toBe("zealot://type/Ticket");
	});

	const bulletWith = (text: string): PMNode => {
		const n = ZealotSchema.nodes;
		const para = n["paragraph"]!.create(null, ZealotSchema.text(text));
		return n["doc"]!.create(null, [n["bullet_list"]!.create(null, [n["list_item"]!.create(null, [para])])]);
	};

	it("'[ ] ' at the start of a list item turns it into an unchecked task", () => {
		const rule = taskItemInputRule(ZealotSchema)!;
		const doc = bulletWith("[ ] ");
		const pos = 3 + 4; // inside the paragraph, after "[ ] "
		const next = apply(rule, doc, pos, "[ ] ", 3);
		const item = next!.doc.child(0).child(0);
		expect(item.attrs.checked).toBe(false);
		expect(item.textContent).toBe("");
	});

	it("'[x] ' makes a checked task and is ignored outside list items", () => {
		const rule = taskItemInputRule(ZealotSchema)!;
		const inList = apply(rule, bulletWith("[x] "), 7, "[x] ", 3);
		expect(inList!.doc.child(0).child(0).attrs.checked).toBe(true);
		const plain = ZealotSchema.nodes["doc"]!.create(null, [
			ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text("[x] ")),
		]);
		expect(apply(rule, plain, 5, "[x] ", 1)).toBeNull();
	});
});

describe("typed view embeds", () => {
	const rule = viewEmbedInputRule(ZealotSchema)!;
	const run = (doc: PMNode, typed: string, pos: number, from: number) => {
		const state = stateAt(doc, pos, []);
		const match = (rule as unknown as { match: RegExp }).match.exec(typed);
		if (!match) return "no match";
		const handler = (rule as unknown as { handler: (s: EditorState, m: RegExpExecArray, a: number, b: number) => unknown }).handler;
		const tr = handler(state, match, from, pos) as import("prosemirror-state").Transaction | null;
		return tr ? state.apply(tr).doc : null;
	};
	const para = (text: string) => ZealotSchema.nodes["paragraph"]!.create(null, ZealotSchema.text(text));
	const docOf = (...blocks: PMNode[]) => ZealotSchema.nodes["doc"]!.create(null, blocks);

	it("turns a line that is only ![[Title]] into an embed at once", () => {
		const typed = "![[AI Queue]]";
		const doc = run(docOf(para(typed)), typed, 1 + typed.length, 1) as PMNode;
		expect(doc.child(0).type.name).toBe("view_embed");
		expect(serializeZealotScript(doc)).toBe(typed);
	});

	it("keeps a |table suffix", () => {
		const typed = "![[AI Queue|table]]";
		const doc = run(docOf(para(typed)), typed, 1 + typed.length, 1) as PMNode;
		expect(serializeZealotScript(doc)).toBe(typed);
	});

	it("does not fire after other text on the line", () => {
		expect(run(docOf(para("see ![[AI Queue]]")), "see ![[AI Queue]]", 18, 1)).toBe("no match");
	});

	it("does not fire where a block cannot go (a list item's first paragraph)", () => {
		const item = ZealotSchema.nodes["list_item"]!.create({ checked: null }, [para("![[AI Queue]]")]);
		const doc = docOf(ZealotSchema.nodes["bullet_list"]!.create(null, [item]));
		expect(run(doc, "![[AI Queue]]", 3 + 13, 3)).toBeNull();
	});
});

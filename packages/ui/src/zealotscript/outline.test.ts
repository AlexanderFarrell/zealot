import { describe, expect, it } from "vitest";
import { EditorState, TextSelection } from "prosemirror-state";
import { baseKeymap, chainCommands } from "prosemirror-commands";
import { liftListItem, sinkListItem, splitListItem } from "prosemirror-schema-list";
import type { Node as PMNode } from "prosemirror-model";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { computeOutlineLabels } from "./outline_numbers";
import { convertListTo } from "./commands";

const parse = (md: string): PMNode => parseZealotScript(ZealotSchema, md);
const roundTrip = (md: string): string => serializeZealotScript(parse(md));
const labels = (doc: PMNode): string[] => computeOutlineLabels(doc).map((l) => l.label);
const itemType = ZealotSchema.nodes["list_item"]!;

// Cursor inside the paragraph whose text starts with `text`.
const stateIn = (doc: PMNode, text: string): EditorState => {
	let at = -1;
	doc.descendants((node, pos) => {
		if (at === -1 && node.type.name === "paragraph" && node.textContent.startsWith(text)) at = pos + 1;
	});
	if (at === -1) throw new Error(`no paragraph ${text}`);
	return EditorState.create({ doc, selection: TextSelection.create(doc, at) });
};

const run = (state: EditorState, cmd: (s: EditorState, d?: (tr: import("prosemirror-state").Transaction) => void) => boolean): EditorState => {
	let next = state;
	cmd(state, (tr) => { next = state.apply(tr); });
	return next;
};

const OUTLINE = ":::outline\n1. Goals\n  1.1 [x] Learn\n  1.2 [ ] Ship\n    1.2.1 Parse\n2. Risks\n:::";

describe("outline lists", () => {
	it("parses to an outline_list, not an admonition", () => {
		const doc = parse(OUTLINE);
		expect(doc.childCount).toBe(1);
		expect(doc.child(0).type.name).toBe("outline_list");
		expect(doc.child(0).childCount).toBe(2);
	});

	it("computes dotted-decimal labels from position", () => {
		expect(labels(parse(OUTLINE))).toEqual(["1.", "1.1", "1.2", "1.2.1", "2."]);
	});

	it("saves the computed numbers, so raw readers can cite 1.2.1", () => {
		const out = roundTrip(OUTLINE);
		expect(out).toContain("\n    1.2.1 Parse\n");
		expect(out).toBe(OUTLINE);
	});

	it("ignores typed numbers and rewrites them from structure", () => {
		const out = roundTrip(":::outline\n7. a\n  9.9 b\n  3 c\n5. d\n:::");
		expect(out).toBe(":::outline\n1. a\n  1.1 b\n  1.2 c\n2. d\n:::");
	});

	it("takes numberless lines and tab nesting", () => {
		expect(roundTrip(":::outline\na\n\tb\nc\n:::")).toBe(":::outline\n1. a\n  1.1 b\n2. c\n:::");
	});

	it("keeps checkbox state through load and save", () => {
		const doc = parse(OUTLINE);
		const checks: Array<boolean | null> = [];
		doc.descendants((n) => { if (n.type.name === "list_item") checks.push(n.attrs.checked); });
		expect(checks).toEqual([null, true, false, null, null]);
	});

	it("keeps an empty outline stable", () => {
		expect(roundTrip(":::outline\n:::")).toBe(":::outline\n1.\n:::");
		expect(roundTrip(":::outline\n1.\n:::")).toBe(":::outline\n1.\n:::");
	});

	it("an item whose text starts with a number survives", () => {
		expect(roundTrip(":::outline\n1. 1.5 million users\n:::")).toBe(":::outline\n1. 1.5 million users\n:::");
	});

	it("leaves bullet and ordered parsing alone", () => {
		expect(roundTrip("1. a\n2. b")).toBe("1. a\n2. b");
		expect(roundTrip("- a\n\t- b")).toBe("- a\n\t- b");
		expect(roundTrip(":::note\ntext\n:::")).toBe(":::note\ntext\n:::");
	});

	it("renumbers when an item is indented, outdented or inserted", () => {
		const doc = parse(":::outline\n1. a\n2. b\n3. c\n:::");
		const indented = run(stateIn(doc, "b"), sinkListItem(itemType));
		expect(labels(indented.doc)).toEqual(["1.", "1.1", "2."]);
		expect(serializeZealotScript(indented.doc)).toBe(":::outline\n1. a\n  1.1 b\n2. c\n:::");

		const back = run(stateIn(indented.doc, "b"), liftListItem(itemType));
		expect(labels(back.doc)).toEqual(["1.", "2.", "3."]);

		const split = run(stateIn(back.doc, "a"), splitListItem(itemType));
		expect(labels(split.doc)).toEqual(["1.", "2.", "3.", "4."]);
	});

	it("numbers three levels deep after indenting twice", () => {
		const doc = parse(":::outline\n1. a\n2. b\n3. c\n:::");
		const once = run(stateIn(doc, "b"), sinkListItem(itemType));
		const twice = run(stateIn(once.doc, "c"), sinkListItem(itemType));
		expect(labels(twice.doc)).toEqual(["1.", "1.1", "1.2"]);
		const deeper = run(stateIn(twice.doc, "c"), sinkListItem(itemType));
		expect(labels(deeper.doc)).toEqual(["1.", "1.1", "1.1.1"]);
		expect(serializeZealotScript(deeper.doc)).toContain("    1.1.1 c");
	});

	it("Enter on an empty last outline item exits the outline", () => {
		const doc = parse(":::outline\n1. a\n2.\n:::");
		let pos = -1;
		doc.descendants((n, p) => { if (n.type.name === "paragraph" && n.content.size === 0) pos = p + 1; });
		const state = EditorState.create({ doc, selection: TextSelection.create(doc, pos) });
		const next = run(state, chainCommands(splitListItem(itemType), baseKeymap["Enter"]!));
		expect(next.doc.lastChild?.type.name).toBe("paragraph");
		expect(next.doc.child(0).childCount).toBe(1);
	});
});

describe("converting lists to and from outline", () => {
	it("bullet to outline keeps text, nesting and checkboxes", () => {
		const doc = parse("- [x] a\n\t- [ ] b\n\t- c\n- d");
		const next = run(stateIn(doc, "b"), convertListTo("outline_list"));
		expect(serializeZealotScript(next.doc)).toBe(":::outline\n1. [x] a\n  1.1 [ ] b\n  1.2 c\n2. d\n:::");
	});

	it("ordered to outline and back to bullet", () => {
		const doc = parse("1. a\n2. b");
		const outlined = run(stateIn(doc, "a"), convertListTo("outline_list"));
		expect(serializeZealotScript(outlined.doc)).toBe(":::outline\n1. a\n2. b\n:::");
		const bullets = run(stateIn(outlined.doc, "a"), convertListTo("bullet_list"));
		expect(serializeZealotScript(bullets.doc)).toBe("- a\n- b");
	});

	it("outline to ordered flattens to per-level numbering", () => {
		const doc = parse(OUTLINE);
		const next = run(stateIn(doc, "Parse"), convertListTo("ordered_list"));
		expect(serializeZealotScript(next.doc)).toBe("1. Goals\n\t1. [x] Learn\n\t2. [ ] Ship\n\t\t1. Parse\n2. Risks");
	});

	it("converting a nested item converts the whole list", () => {
		const doc = parse("- a\n\t- b\n- c");
		const next = run(stateIn(doc, "b"), convertListTo("outline_list"));
		expect(labels(next.doc)).toEqual(["1.", "1.1", "2."]);
	});

	it("outside a list the command starts one", () => {
		const doc = parse("plain");
		const next = run(stateIn(doc, "plain"), convertListTo("outline_list"));
		expect(serializeZealotScript(next.doc)).toBe(":::outline\n1. plain\n:::");
	});

	it("keeps the cursor where it was", () => {
		const doc = parse("- a\n- b");
		const before = stateIn(doc, "b");
		const next = run(before, convertListTo("outline_list"));
		expect(next.selection.from).toBe(before.selection.from);
	});
});

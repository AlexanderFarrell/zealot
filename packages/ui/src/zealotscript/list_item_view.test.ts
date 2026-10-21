import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { TaskListItemView } from "./zealotscript_editor";

// Z259: in the macOS desktop app (WKWebView), a block wrapper between <li> and its
// paragraph made WebKit drop the list marker while typing until the next re-layout.
// Plain items must keep their paragraphs directly inside the <li>; only task items
// wrap their content, and they draw a checkbox instead of a marker.

let view: EditorView | null = null;

afterEach(() => {
	view?.destroy();
	view = null;
});

function render(md: string): HTMLElement {
	const mount = document.createElement("div");
	document.body.appendChild(mount);
	view = new EditorView(mount, {
		state: EditorState.create({ doc: parseZealotScript(ZealotSchema, md) }),
		nodeViews: { list_item: (node, v, getPos) => new TaskListItemView(node, v, getPos) },
	});
	return view.dom;
}

function typeInto(text: string, at: number): void {
	view!.dispatch(view!.state.tr.insertText(text, at));
}

const childTags = (li: Element): string[] => Array.from(li.children).map((c) => c.tagName);

describe("Z259: list items keep a marker-friendly structure", () => {
	it.each([
		["unordered", "- First item\n- Second item", "UL"],
		["ordered", "1. First item\n2. Second item", "OL"],
	])("%s items hold their paragraph directly", (_kind, md, listTag) => {
		const dom = render(md);
		const items = dom.querySelectorAll(`${listTag} > li`);
		expect(items).toHaveLength(2);
		for (const li of items) {
			expect(childTags(li)).toEqual(["P"]);
			expect(li.querySelector(".zealot-task-content")).toBeNull();
		}
	});

	it("keeps that structure while typing into the last item", () => {
		const dom = render("- First item\n- Second item");
		// End of the last item's text: before the closing paragraph, item and list tokens.
		const endOfText = () => view!.state.doc.content.size - 3;
		for (const word of [" and", " more", " words"]) typeInto(word, endOfText());
		const items = dom.querySelectorAll("ul > li");
		expect(items[1]!.textContent).toBe("Second item and more words");
		for (const li of items) expect(childTags(li)).toEqual(["P"]);
	});

	it("nested items keep their paragraph and sublist directly in the item", () => {
		const dom = render("- Parent\n\t- Child");
		const parent = dom.querySelector("ul > li")!;
		expect(childTags(parent)).toEqual(["P", "UL"]);
	});

	it("task items still wrap their content beside the checkbox", () => {
		const dom = render("- [ ] Open task\n- [x] Done task");
		const items = dom.querySelectorAll("li.zealot-task-item");
		expect(items).toHaveLength(2);
		for (const li of items) {
			expect(childTags(li)).toEqual(["INPUT", "SPAN"]);
			expect(li.querySelector(":scope > .zealot-task-content > p")).not.toBeNull();
		}
		expect(items[1]!.getAttribute("data-checked")).toBe("true");
	});

	it("rebuilds an item that becomes a task, and one that stops being one", () => {
		const dom = render("- Plain item");
		const itemPos = 1;
		view!.dispatch(view!.state.tr.setNodeMarkup(itemPos, undefined, { checked: false }));
		expect(childTags(dom.querySelector("li")!)).toEqual(["INPUT", "SPAN"]);
		view!.dispatch(view!.state.tr.setNodeMarkup(itemPos, undefined, { checked: null }));
		expect(childTags(dom.querySelector("li")!)).toEqual(["P"]);
	});
});

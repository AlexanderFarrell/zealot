import { TextSelection, type EditorState, type Transaction } from "prosemirror-state";
import { isInTable } from "prosemirror-tables";
import { wrapInList } from "prosemirror-schema-list";
import type { Node as PMNode } from "prosemirror-model";
import { extractYouTubeVideoId } from "./parse/parse_youtube";

export { extractYouTubeVideoId, isInTable };

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

export const insertTable: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const tableCell = schema.nodes["table_cell"];
	const tableHeader = schema.nodes["table_header"];
	const tableRow = schema.nodes["table_row"];
	const table = schema.nodes["table"];
	if (!paragraph || !tableCell || !tableHeader || !tableRow || !table) return false;

	const makeCell = (header: boolean, text?: string) => {
		const p = text ? paragraph.createAndFill({}, schema.text(text))! : paragraph.create();
		return header
			? tableHeader.createAndFill({}, [p])!
			: tableCell.createAndFill({}, [p])!;
	};

	const headerRow = tableRow.createAndFill({}, [makeCell(true, "Column 1"), makeCell(true, "Column 2")])!;
	const dataRow = tableRow.createAndFill({}, [makeCell(false), makeCell(false)])!;
	const node = table.createAndFill({}, [headerRow, dataRow])!;

	// Tables are isolating so replaceSelectionWith fails inside a paragraph.
	// Insert after the top-level block that contains the cursor.
	const { $from } = state.selection;
	const insertPos = $from.depth >= 1
		? $from.after(1)
		: $from.pos;
	const tr = state.tr.insert(insertPos, node).scrollIntoView();
	if (dispatch) dispatch(tr);
	return true;
};

export const insertYoutubeEmbed = (videoId: string): PMCommand => (state, dispatch) => {
	const youtubeEmbed = state.schema.nodes["youtube_embed"];
	if (!youtubeEmbed) return false;
	const node = youtubeEmbed.create({ videoId });
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertAdmonition = (kind: string): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const admonition = schema.nodes["admonition"];
	if (!paragraph || !admonition) return false;

	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = admonition.createAndFill({ kind }, [p])!;
	const tr = state.tr.replaceSelectionWith(node).scrollIntoView();
	if (dispatch) dispatch(tr);
	return true;
};

export const insertMermaidBlock: PMCommand = (state, dispatch) => {
	const codeBlock = state.schema.nodes["code_block"];
	if (!codeBlock) return false;
	const node = codeBlock.create({ language: "mermaid" });
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertDetails = (summary = "Details"): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const details = schema.nodes["details"];
	if (!paragraph || !details) return false;
	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = details.createAndFill({ summary }, [p]);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertSpoiler: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const spoiler = schema.nodes["spoiler"];
	if (!paragraph || !spoiler) return false;
	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = spoiler.createAndFill({}, [p]);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertColumns = (numCols: 2 | 3 | 4 = 2): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const column = schema.nodes["column"];
	const columns = schema.nodes["columns"];
	if (!paragraph || !column || !columns) return false;
	const cols = Array.from({ length: numCols }, () => column.createAndFill({}, [paragraph.create()])!);
	const node = columns.createAndFill({}, cols);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertTabs = (titles = ["Tab 1", "Tab 2"]): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const tab = schema.nodes["tab"];
	const tabs = schema.nodes["tabs"];
	if (!paragraph || !tab || !tabs) return false;
	const tabNodes = titles.map((title) => tab.createAndFill({ title }, [paragraph.create()])!);
	const node = tabs.createAndFill({}, tabNodes);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

// Enter on an empty last line of a code block leaves the block, like pressing Enter
// twice in a list. Without it the only way out was Mod-Enter, which nobody discovers.
export const exitCodeBlockOnBlankLine: PMCommand = (state, dispatch) => {
	const { $from, empty } = state.selection;
	if (!empty || !$from.parent.type.spec.code) return false;
	if ($from.parentOffset !== $from.parent.content.size) return false;
	if (!$from.parent.textContent.endsWith("\n")) return false;

	const paragraph = state.schema.nodes["paragraph"];
	if (!paragraph) return false;
	const container = $from.node($from.depth - 1);
	const index = $from.index($from.depth - 1);
	if (!container.canReplaceWith(index + 1, index + 1, paragraph)) return false;

	if (dispatch) {
		const blockEnd = $from.after();
		const tr = state.tr.delete($from.pos - 1, $from.pos);
		const insertAt = blockEnd - 1;
		tr.insert(insertAt, paragraph.create());
		tr.setSelection(TextSelection.create(tr.doc, insertAt + 1));
		dispatch(tr.scrollIntoView());
	}
	return true;
};

const LIST_TYPES = new Set(["bullet_list", "ordered_list", "outline_list"]);

// Positions of every list in the tree rooted at `list`, including the root. Changing a list's
// type does not change its size, so these positions stay valid while the type is swapped.
const collectListPositions = (list: PMNode, pos: number, out: number[]): void => {
	out.push(pos);
	list.forEach((item, itemOffset) => {
		item.forEach((child, childOffset) => {
			if (LIST_TYPES.has(child.type.name)) collectListPositions(child, pos + 1 + itemOffset + 1 + childOffset, out);
		});
	});
};

// With the cursor in a list, converts the whole list (top-level list and every nested one) to
// `target`, keeping item text, nesting and checkbox state. Outside a list, wraps the block.
// A nested outline_list must sit in an outline_list for the numbers to read right, which is why
// the whole tree changes together.
export const convertListTo = (target: "bullet_list" | "ordered_list" | "outline_list"): PMCommand => (state, dispatch) => {
	const targetType = state.schema.nodes[target];
	if (!targetType) return false;
	const { $from } = state.selection;

	let depth = -1;
	for (let d = $from.depth; d > 0; d--) {
		if (LIST_TYPES.has($from.node(d).type.name)) { depth = d; break; }
	}
	if (depth === -1) return wrapInList(targetType)(state, dispatch);

	// Climb to the outermost list of the chain list > list_item > list > ...
	while (depth >= 2 && $from.node(depth - 1).type.name === "list_item" && LIST_TYPES.has($from.node(depth - 2).type.name)) {
		depth -= 2;
	}
	const root = $from.node(depth);
	if (root.type === targetType) return true;

	if (dispatch) {
		const positions: number[] = [];
		collectListPositions(root, $from.before(depth), positions);
		const tr = state.tr;
		for (const pos of positions) tr.setNodeMarkup(pos, targetType);
		dispatch(tr.scrollIntoView());
	}
	return true;
};

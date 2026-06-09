import type { EditorState, Transaction } from "prosemirror-state";
import type { Node as PMNode } from "prosemirror-model";
import { extractYouTubeVideoId } from "./parse/parse_youtube";

export { extractYouTubeVideoId };

function findTableContext(state: EditorState): { tablePos: number; tableNode: PMNode; rowIndex: number; colIndex: number } | null {
	const { $from } = state.selection;
	const { schema } = state;
	const tableType = schema.nodes["table"];
	const rowType = schema.nodes["table_row"];
	if (!tableType || !rowType) return null;

	let tablePos = -1;
	let tableNode: PMNode | null = null;
	let rowIndex = -1;
	let colIndex = -1;

	for (let d = $from.depth; d >= 0; d--) {
		const node = $from.node(d);
		if (node.type === tableType) {
			tablePos = $from.before(d);
			tableNode = node;
			break;
		}
	}
	if (!tableNode || tablePos < 0) return null;

	tableNode.forEach((row, _offset, ri) => {
		if (row.type !== rowType) return;
		row.forEach((_cell, _o, ci) => {
			const cellStart = tablePos + 1 + tableNode!.child(0 as number).nodeSize * ri;
			if ($from.pos >= cellStart) {
				rowIndex = ri;
				colIndex = ci;
			}
		});
	});

	return { tablePos, tableNode, rowIndex, colIndex };
}

export function isInTable(state: EditorState): boolean {
	const { schema } = state;
	const tableType = schema.nodes["table"];
	if (!tableType) return false;
	const { $from } = state.selection;
	for (let d = $from.depth; d >= 0; d--) {
		if ($from.node(d).type === tableType) return true;
	}
	return false;
}

export const addTableRowAfter: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const tableType = schema.nodes["table"];
	const rowType = schema.nodes["table_row"];
	const cellType = schema.nodes["table_cell"];
	const paragraph = schema.nodes["paragraph"];
	if (!tableType || !rowType || !cellType || !paragraph) return false;

	const { $from } = state.selection;
	let rowDepth = -1;
	for (let d = $from.depth; d >= 0; d--) {
		if ($from.node(d).type === rowType) { rowDepth = d; break; }
	}
	if (rowDepth < 0) return false;

	const existingRow = $from.node(rowDepth);
	const newCells = Array.from({ length: existingRow.childCount }, () =>
		cellType.createAndFill({}, [paragraph.create()])!
	);
	const newRow = rowType.createAndFill({}, newCells)!;
	const rowEnd = $from.after(rowDepth);

	if (dispatch) dispatch(state.tr.insert(rowEnd, newRow).scrollIntoView());
	return true;
};

export const removeTableRow: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const tableType = schema.nodes["table"];
	const rowType = schema.nodes["table_row"];
	if (!tableType || !rowType) return false;

	const { $from } = state.selection;
	let rowDepth = -1;
	for (let d = $from.depth; d >= 0; d--) {
		if ($from.node(d).type === rowType) { rowDepth = d; break; }
	}
	if (rowDepth < 0) return false;

	// Don't remove if it's the only row.
	const tableNode = $from.node(rowDepth - 1);
	if (tableNode.childCount <= 1) return false;

	const rowStart = $from.before(rowDepth);
	const rowEnd = $from.after(rowDepth);
	if (dispatch) dispatch(state.tr.delete(rowStart, rowEnd).scrollIntoView());
	return true;
};

export const addTableColumnAfter: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const tableType = schema.nodes["table"];
	const rowType = schema.nodes["table_row"];
	const cellType = schema.nodes["table_cell"];
	const headerType = schema.nodes["table_header"];
	const paragraph = schema.nodes["paragraph"];
	if (!tableType || !rowType || !cellType || !headerType || !paragraph) return false;

	const { $from } = state.selection;

	// Find the column index of the current cell.
	let colDepth = -1;
	for (let d = $from.depth; d >= 0; d--) {
		const nodeType = $from.node(d).type;
		if (nodeType === cellType || nodeType === headerType) { colDepth = d; break; }
	}
	if (colDepth < 0) return false;

	let tableDepth = -1;
	for (let d = colDepth - 1; d >= 0; d--) {
		if ($from.node(d).type === tableType) { tableDepth = d; break; }
	}
	if (tableDepth < 0) return false;

	const tableNode = $from.node(tableDepth);
	const tableStart = $from.start(tableDepth);

	// Find current col index in its row.
	const rowDepth = colDepth - 1;
	const rowNode = $from.node(rowDepth);
	let colIndex = -1;
	let offset = 0;
	rowNode.forEach((cell, o, i) => {
		if ($from.pos >= $from.start(rowDepth) + o && $from.pos <= $from.start(rowDepth) + o + cell.nodeSize) {
			colIndex = i;
		}
		offset += cell.nodeSize;
	});
	if (colIndex < 0) return false;

	const tr = state.tr;
	let rowOffset = 0;

	tableNode.forEach((row, _ro, ri) => {
		if (row.type !== rowType) return;
		const isHeader = ri === 0 && row.firstChild?.type === headerType;
		const newCell = isHeader
			? headerType.createAndFill({}, [paragraph.create()])!
			: cellType.createAndFill({}, [paragraph.create()])!;

		// Position after the cell at colIndex in this row.
		let insertPos = tableStart + 1 + rowOffset;
		let cellOffset = 0;
		row.forEach((cell, co, ci) => {
			if (ci === colIndex) insertPos = tableStart + 1 + rowOffset + 1 + cellOffset + cell.nodeSize;
			cellOffset += cell.nodeSize;
		});
		tr.insert(tr.mapping.map(insertPos), newCell);
		rowOffset += row.nodeSize;
	});

	if (dispatch) dispatch(tr.scrollIntoView());
	return true;
};

export const removeTableColumn: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const tableType = schema.nodes["table"];
	const rowType = schema.nodes["table_row"];
	const cellType = schema.nodes["table_cell"];
	const headerType = schema.nodes["table_header"];
	if (!tableType || !rowType || !cellType || !headerType) return false;

	const { $from } = state.selection;

	let colDepth = -1;
	for (let d = $from.depth; d >= 0; d--) {
		const nodeType = $from.node(d).type;
		if (nodeType === cellType || nodeType === headerType) { colDepth = d; break; }
	}
	if (colDepth < 0) return false;

	let tableDepth = -1;
	for (let d = colDepth - 1; d >= 0; d--) {
		if ($from.node(d).type === tableType) { tableDepth = d; break; }
	}
	if (tableDepth < 0) return false;

	const tableNode = $from.node(tableDepth);

	// Don't remove if only one column.
	if ((tableNode.firstChild?.childCount ?? 0) <= 1) return false;

	const tableStart = $from.start(tableDepth);
	const rowDepth = colDepth - 1;
	const rowNode = $from.node(rowDepth);
	let colIndex = -1;
	rowNode.forEach((cell, co) => {
		if ($from.pos >= $from.start(rowDepth) + co && $from.pos <= $from.start(rowDepth) + co + cell.nodeSize) {
			colIndex = rowNode.childCount - 1;  // will be overwritten
		}
	});
	// Recompute precisely.
	colIndex = -1;
	rowNode.forEach((cell, co, ci) => {
		if ($from.pos >= $from.start(rowDepth) + 1 + co && $from.pos < $from.start(rowDepth) + 1 + co + cell.nodeSize) {
			colIndex = ci;
		}
	});
	if (colIndex < 0) return false;

	const tr = state.tr;
	let rowOffset = 0;

	tableNode.forEach((row, _ro) => {
		if (row.type !== rowType) return;
		let cellOffset = 0;
		row.forEach((cell, co, ci) => {
			if (ci === colIndex) {
				const cellStart = tr.mapping.map(tableStart + 1 + rowOffset + 1 + cellOffset);
				tr.delete(cellStart, cellStart + tr.mapping.map(tableStart + 1 + rowOffset + 1 + cellOffset + cell.nodeSize) - cellStart);
			}
			cellOffset += cell.nodeSize;
		});
		rowOffset += row.nodeSize;
	});

	if (dispatch) dispatch(tr.scrollIntoView());
	return true;
};

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

export const insertTable: PMCommand = (state, dispatch) => {
	const { schema, selection } = state;
	const paragraph = schema.nodes["paragraph"];
	const tableCell = schema.nodes["table_cell"];
	const tableHeader = schema.nodes["table_header"];
	const tableRow = schema.nodes["table_row"];
	const table = schema.nodes["table"];
	if (!paragraph || !tableCell || !tableHeader || !tableRow || !table) return false;

	const makeCell = (header: boolean, text: string) => {
		const p = paragraph.createAndFill({}, schema.text(text))!;
		return header
			? tableHeader.createAndFill({}, [p])!
			: tableCell.createAndFill({}, [p])!;
	};

	const headerRow = tableRow.createAndFill({}, [makeCell(true, "Column 1"), makeCell(true, "Column 2")])!;
	const dataRow = tableRow.createAndFill({}, [makeCell(false, ""), makeCell(false, "")])!;
	const node = table.createAndFill({}, [headerRow, dataRow])!;

	// Insert after the current top-level block so we never replace inside an inline context.
	const { $from } = selection;
	const insertPos = $from.end($from.depth === 0 ? 0 : 1);
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

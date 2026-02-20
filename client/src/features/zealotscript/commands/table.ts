import type {Node as PMNode, Schema} from "prosemirror-model";
import type { ZCommandBlockInfo } from "./types";
import { parseInlineNodes } from "../parse/parse_inline";
import { escapeTableCell, isMarkdownTableSeparator, splitTableRow } from "../table_utils";

const createCell = (schema: Schema, text: string, header: boolean): PMNode => {
	const paragraph = schema.nodes.paragraph.create(null, parseInlineNodes(schema, text));
	const cellType = header ? schema.nodes.table_header : schema.nodes.table_cell;
	return cellType.create(null, [paragraph]);
}

const createRow = (schema: Schema, cells: string[], header: boolean): PMNode => {
	return schema.nodes.table_row.create(null, cells.map((cell) => createCell(schema, cell, header)));
}

const parseTableBlock: ZCommandBlockInfo["parse"] = (schema, lines, startIndex) => {
	const contentLines: string[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		if (line.trim() === ":::") break;
		contentLines.push(line);
		index++;
	}

	const rows: PMNode[] = [];
	if (contentLines.length > 0) {
		const firstRowCells = splitTableRow(contentLines[0]);
		const hasMarkdownHeader =
			contentLines.length > 1 &&
			firstRowCells.length > 0 &&
			isMarkdownTableSeparator(contentLines[1], firstRowCells.length);

		let rowStartIndex = 0;
		let expectedColumns: number | undefined = undefined;

		if (hasMarkdownHeader) {
			rows.push(createRow(schema, firstRowCells, true));
			rowStartIndex = 2;
			expectedColumns = firstRowCells.length;
		}

		for (let i = rowStartIndex; i < contentLines.length; i++) {
			const cells = splitTableRow(contentLines[i]);
			if (cells.length === 0) continue;
			const normalized = expectedColumns
				? Array.from({ length: expectedColumns }, (_, columnIndex) => cells[columnIndex] ?? "")
				: cells;
			rows.push(createRow(schema, normalized, false));
		}
	}

	if (rows.length === 0) {
		rows.push(createRow(schema, [""], false));
	}

	const table = schema.nodes.table.create(null, rows);
	if (!table) return null;

	const hasClosingFence = index < lines.length && lines[index].trim() === ":::";
	return {
		node: table,
		linesConsumed: index - startIndex + (hasClosingFence ? 1 : 0)
	};
}

const serializeTable: ZCommandBlockInfo["serialize"] = (node) => {
	if (node.type.name !== "table") return null;
	const lines: string[] = [];
	lines.push(":::table");
	node.forEach((row) => {
		const cells: string[] = [];
		row.forEach((cell) => {
			const text = cell.textContent || "";
			cells.push(escapeTableCell(text));
		});
		lines.push(cells.join(" | "));
	});
	lines.push(":::");
	return lines.join("\n");
}

export const TableInfo: ZCommandBlockInfo = {
	name: "table",
	parse: parseTableBlock,
	serialize: serializeTable,
}

export default TableInfo;

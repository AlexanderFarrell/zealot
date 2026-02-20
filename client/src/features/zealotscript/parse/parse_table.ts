import type {Node as PMNode, Schema} from "prosemirror-model";
import { parseInlineNodes } from "./parse_inline";
import { isMarkdownTableSeparator, splitTableRow } from "../table_utils";

const createTableCell = (schema: Schema, text: string, header: boolean): PMNode => {
	const paragraph = schema.nodes.paragraph.create(null, parseInlineNodes(schema, text));
	const cellType = header ? schema.nodes.table_header : schema.nodes.table_cell;
	return cellType.create(null, [paragraph]);
}

const createTableRow = (schema: Schema, cells: string[], header: boolean): PMNode => {
	const mapped = cells.map((cell) => createTableCell(schema, cell, header));
	return schema.nodes.table_row.create(null, mapped);
}

const nextNonEmptyLineIndex = (lines: string[], startIndex: number): number => {
	let index = startIndex;
	while (index < lines.length && lines[index].trim().length === 0) {
		index++;
	}
	return index;
}

export const parseMarkdownTable = (schema: Schema, lines: string[], startIndex: number) => {
	if (startIndex >= lines.length) return null;

	const headerLine = lines[startIndex];
	if (!headerLine.includes("|")) return null;

	const headerCells = splitTableRow(headerLine);
	if (headerCells.length === 0) return null;

	const separatorIndex = nextNonEmptyLineIndex(lines, startIndex + 1);
	if (separatorIndex >= lines.length) return null;
	const separatorLine = lines[separatorIndex];
	if (!isMarkdownTableSeparator(separatorLine, headerCells.length)) return null;

	const rows: PMNode[] = [createTableRow(schema, headerCells, true)];
	let index = separatorIndex + 1;

	while (index < lines.length) {
		if (lines[index].trim().length === 0) {
			const peekIndex = nextNonEmptyLineIndex(lines, index + 1);
			if (peekIndex >= lines.length) {
				index = peekIndex;
				break;
			}
			const peekLine = lines[peekIndex];
			if (!peekLine.includes("|")) break;
			if (isMarkdownTableSeparator(peekLine, headerCells.length)) break;
			index = peekIndex;
		}

		const line = lines[index];
		if (!line.includes("|")) break;
		if (isMarkdownTableSeparator(line, headerCells.length)) break;

		const parsed = splitTableRow(line);
		if (parsed.length === 0) break;
		const normalized = headerCells.map((_, columnIndex) => parsed[columnIndex] ?? "");
		rows.push(createTableRow(schema, normalized, false));
		index++;
	}

	const table = schema.nodes.table.create(null, rows);
	return {
		node: table,
		linesConsumed: index - startIndex
	};
}

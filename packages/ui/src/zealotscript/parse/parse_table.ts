import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseInlineNodes } from "./parse_inline";
import { isMarkdownTableSeparator, splitTableRow } from "../table_utils";

const createTableCell = (schema: Schema, text: string, header: boolean): PMNode => {
	const paragraph = schema.nodes["paragraph"];
	if (!paragraph) throw new Error("Schema missing paragraph node");
	const p = paragraph.create(null, parseInlineNodes(schema, text));
	const cellType = header ? schema.nodes["table_header"] : schema.nodes["table_cell"];
	if (!cellType) throw new Error("Schema missing table cell node");
	return cellType.create(null, [p]);
};

const createTableRow = (schema: Schema, cells: string[], header: boolean): PMNode => {
	const tableRow = schema.nodes["table_row"];
	if (!tableRow) throw new Error("Schema missing table_row node");
	const mapped = cells.map((cell) => createTableCell(schema, cell, header));
	return tableRow.create(null, mapped);
};

export const parseFencedTable = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine || !/^:::table\s*$/.test(openLine)) return null;

	const contentLines: string[] = [];
	let index = startIndex + 1;
	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (line.trim() === ":::") { index++; break; }
		contentLines.push(line);
		index++;
	}
	const linesConsumed = index - startIndex;

	const dataLines = contentLines.filter((l) => l.trim().length > 0);
	if (dataLines.length === 0) return null;

	const table = schema.nodes["table"];
	if (!table) return null;

	// If the second non-empty line is a markdown separator, treat the first as a header row.
	const hasHeader =
		dataLines.length >= 2 &&
		isMarkdownTableSeparator(dataLines[1]!, splitTableRow(dataLines[0]!).length);

	const rows: PMNode[] = [];
	const linesToParse = hasHeader
		? [dataLines[0]!, ...dataLines.slice(2)]
		: dataLines;

	linesToParse.forEach((line, i) => {
		const isHeader = hasHeader && i === 0;
		const cells = splitTableRow(line);
		if (cells.length === 0) return;
		rows.push(createTableRow(schema, cells, isHeader));
	});

	if (rows.length === 0) return null;
	return { node: table.create(null, rows), linesConsumed };
};

const nextNonEmptyLineIndex = (lines: string[], startIndex: number): number => {
	let index = startIndex;
	while (index < lines.length && (lines[index]?.trim().length ?? 0) === 0) index++;
	return index;
};

export const parseMarkdownTable = (schema: Schema, lines: string[], startIndex: number) => {
	if (startIndex >= lines.length) return null;

	const headerLine = lines[startIndex];
	if (!headerLine?.includes("|")) return null;

	const headerCells = splitTableRow(headerLine);
	if (headerCells.length === 0) return null;

	const separatorIndex = nextNonEmptyLineIndex(lines, startIndex + 1);
	if (separatorIndex >= lines.length) return null;
	const separatorLine = lines[separatorIndex];
	if (!separatorLine || !isMarkdownTableSeparator(separatorLine, headerCells.length)) return null;

	const rows: PMNode[] = [createTableRow(schema, headerCells, true)];
	let index = separatorIndex + 1;

	while (index < lines.length) {
		const currentLine = lines[index];
		if (!currentLine || currentLine.trim().length === 0) {
			const peekIndex = nextNonEmptyLineIndex(lines, index + 1);
			if (peekIndex >= lines.length) { index = peekIndex; break; }
			const peekLine = lines[peekIndex];
			if (!peekLine?.includes("|")) break;
			if (isMarkdownTableSeparator(peekLine, headerCells.length)) break;
			index = peekIndex;
		}

		const line = lines[index];
		if (!line?.includes("|")) break;
		if (isMarkdownTableSeparator(line, headerCells.length)) break;

		const parsed = splitTableRow(line);
		if (parsed.length === 0) break;
		const normalized = headerCells.map((_, columnIndex) => parsed[columnIndex] ?? "");
		rows.push(createTableRow(schema, normalized, false));
		index++;
	}

	const table = schema.nodes["table"];
	if (!table) throw new Error("Schema missing table node");
	return { node: table.create(null, rows), linesConsumed: index - startIndex };
};

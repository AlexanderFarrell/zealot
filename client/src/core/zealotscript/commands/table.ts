import type {Node as PMNode, Schema} from "prosemirror-model";
import type { ZCommandBlockInfo } from "./types";

const splitTableRow = (line: string): string[] => {
	const cells: string[] = [];
	let current_cell = "";
	let escaped = false;

	for (const c of line) {
		if (escaped) {
			current_cell += c;
			escaped = false;
			continue;
		}
		if (c === "\\") {
			escaped = true;
			continue;
		}
		if (c === "|") {
			cells.push(current_cell.trim());
			current_cell = "";
			continue;
		}
		current_cell += c;
	}

	// Push last one
	cells.push(current_cell.trim());
	return cells.map((c) => c.replace(/\\\|/g, "|"));
}

const escapeTableCell = (text: string): string => {
	return text.replace(/\|/g, "\\|");
}

const parseTableBlock: ZCommandBlockInfo["parse"] = (schema, lines, startIndex) => {
	const rows: PMNode[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		if (line.trim() === ":::") break;

		const cells = splitTableRow(line);
		const cellNodes = cells.map((c) => 
			schema.nodes.table_cell.createAndFill({}, schema.nodes.paragraph.createAndFill({}, schema.text(c)!))
		);

		// @ts-ignore
		const row = schema.nodes.table_row.createAndFill({}, cellNodes)!;
		rows.push(row);
		index++;
	}

	const table = schema.nodes.table.createAndFill({}, rows);
	if (!table) return null;

	return {node: table, linesConsumed: index - startIndex + 1};
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
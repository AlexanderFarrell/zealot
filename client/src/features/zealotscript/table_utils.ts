export const splitTableRow = (line: string): string[] => {
	const trimmed = line.trim();
	if (trimmed.length === 0) return [];

	let work = trimmed;
	if (work.startsWith("|")) work = work.slice(1);
	if (work.endsWith("|")) work = work.slice(0, -1);

	const cells: string[] = [];
	let current = "";
	let escaped = false;

	for (const char of work) {
		if (escaped) {
			current += char;
			escaped = false;
			continue;
		}

		if (char === "\\") {
			escaped = true;
			continue;
		}

		if (char === "|") {
			cells.push(current.trim());
			current = "";
			continue;
		}

		current += char;
	}

	if (escaped) current += "\\";
	cells.push(current.trim());
	return cells;
}

export const isMarkdownTableSeparator = (line: string, expectedColumns?: number): boolean => {
	const cells = splitTableRow(line);
	if (cells.length === 0) return false;
	if (expectedColumns !== undefined && cells.length !== expectedColumns) return false;
	return cells.every((cell) => /^:?-{3,}:?$/.test(cell.trim()));
}

export const escapeTableCell = (text: string): string => {
	return text.replace(/\\/g, "\\\\").replace(/\|/g, "\\|");
}

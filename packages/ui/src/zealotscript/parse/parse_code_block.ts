import type { Node as PMNode, Schema } from "prosemirror-model";

export const parseCodeBlock = (schema: Schema, lines: string[], startIndex: number) => {
	const startLine = lines[startIndex];
	if (!startLine) return null;
	// A fence is three or more backticks or tildes, as the server reads it (Saved Views). Backtick
	// fences cannot carry backticks in the info string. A plain ``` fence keeps the loose closing
	// rule documents were written under; any other fence closes only on its own marker, as long.
	const openMatch = /^[ \t]*(`{3,}|~{3,})(.*)$/.exec(startLine);
	if (!openMatch) return null;
	const fence = openMatch[1] ?? "```";
	if (fence.startsWith("`") && (openMatch[2] ?? "").includes("`")) return null;

	const language = (openMatch[2] ?? "").trim();
	const plain = fence === "```";
	const marker = fence.charAt(0);
	const closes = (line: string): boolean => {
		if (plain) return /^[ \t]*```/.test(line);
		const trimmed = line.trim();
		return trimmed.length >= fence.length && [...trimmed].every((c) => c === marker);
	};
	let index = startIndex + 1;
	const contentLines: string[] = [];

	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (closes(line)) { index++; break; }
		contentLines.push(line);
		index++;
	}

	const codeBlock = schema.nodes["code_block"];
	if (!codeBlock) return null;

	const content = contentLines.join("\n");
	const node: PMNode =
		content.length === 0
			? codeBlock.create({ language, fence })
			: codeBlock.create({ language, fence }, schema.text(content));

	return { node, linesConsumed: index - startIndex };
};

import type { Node as PMNode, Schema } from "prosemirror-model";

export const parseMathBlock = (schema: Schema, lines: string[], startIndex: number): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	if (!/^:::math\s*$/.test(openLine)) return null;

	const mathBlock = schema.nodes["math_block"];
	if (!mathBlock) return null;

	const contentLines: string[] = [];
	let index = startIndex + 1;
	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (line.trim() === ":::") { index++; break; }
		contentLines.push(line);
		index++;
	}

	const src = contentLines.join("\n");
	const node = mathBlock.create({ src });
	return { node, linesConsumed: index - startIndex };
};

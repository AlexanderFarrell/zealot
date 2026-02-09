import type {Node as PMNode, Schema} from "prosemirror-model";

export const parseCodeBlock = (schema: Schema, lines: string[], startIndex: number) => {
	const startLine = lines[startIndex];
	if (!/^[ \t]*```/.test(startLine)) return null;

	let index = startIndex + 1;
	const contentLines: string[] = [];

	while (index < lines.length) {
		const line = lines[index];
		if (/^[ \t]*```/.test(line)) {
			index++;
			break;
		}
		contentLines.push(line);
		index++;
	}

	const content = contentLines.join('\n');
	const node = 
		content.length === 0
			? schema.nodes.code_block.create()
			: schema.nodes.code_block.create(null, schema.text(content));

	return {node, linesConsumed: index - startIndex};
}
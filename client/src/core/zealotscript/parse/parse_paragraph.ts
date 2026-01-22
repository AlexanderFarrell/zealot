import type {Node as PMNode, Schema} from "prosemirror-model";
import { parseInline } from "./parse_inline";

export const parseParagraphs = (schema: Schema, lines: string[]): PMNode[] => {
	const blocks: PMNode[] = [];
	const buffer: string[] = [];

	const flush = () => {
		if (buffer.length === 0) return;
		const text = buffer.join("\n").trimEnd();
		if (text.trim().length > 0) {
			blocks.push(parseInline(schema, text));
		} 
		buffer.length = 0;
	}

	for (const line of lines) {
		if (line.trim() === "") {
			flush();
			// Blank line
			blocks.push(schema.nodes.paragraph.create())
		} else {
			buffer.push(line);
		}
	}

	flush();
	return blocks;
}

export const makeParagraph = (schema: Schema, text: string) => {
	if (text.trim().length === 0) return schema.nodes.paragraph.create();
	return schema.nodes.paragraph.create(null, schema.text(text));
}
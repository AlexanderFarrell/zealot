import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseInline, parseInlineNodes } from "./parse_inline";

export const parseParagraphs = (schema: Schema, lines: string[]): PMNode[] => {
	const paragraph = schema.nodes["paragraph"];
	if (!paragraph) throw new Error("Schema missing paragraph node");

	const blocks: PMNode[] = [];
	const buffer: string[] = [];

	const flush = () => {
		if (buffer.length === 0) return;
		const text = buffer.join("\n").trimEnd();
		if (text.trim().length > 0) {
			blocks.push(parseInline(schema, text));
		}
		buffer.length = 0;
	};

	for (const line of lines) {
		if (line.trim() === "") {
			flush();
			blocks.push(paragraph.create());
		} else {
			buffer.push(line);
		}
	}

	flush();
	return blocks;
};

export const makeParagraph = (schema: Schema, text: string): PMNode => {
	const paragraph = schema.nodes["paragraph"];
	if (!paragraph) throw new Error("Schema missing paragraph node");
	if (text.trim().length === 0) return paragraph.create();
	return paragraph.create(null, parseInlineNodes(schema, text));
};

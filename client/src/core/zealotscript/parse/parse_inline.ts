import type {Node as PMNode, Schema} from "prosemirror-model";

export type InlineToken = {text: string};

export const parseInline = (schema: Schema, text: string): PMNode => {
	// Minimal 
	return schema.nodes.paragraph.createAndFill({}, schema.text(text))!;
}
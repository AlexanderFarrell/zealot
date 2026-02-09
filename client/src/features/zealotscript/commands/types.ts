import type {Node as PMNode, Schema} from "prosemirror-model";

export type ZCommandBlockResult = {
	node: PMNode,
	linesConsumed: number,
}

export type ZCommandBlockInfo = {
	name: string;
	parse: (schema: Schema, lines: string[], startIndex: number, args: string[]) => ZCommandBlockResult | null;
	serialize: (node: PMNode) => string | null;
}
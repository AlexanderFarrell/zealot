import type {Node as PMNode} from "prosemirror-model";
import { ZCommandBlocks } from "./commands/commands";

const serializeCommandBlock = (node: PMNode): string | null => {
	for (const block of ZCommandBlocks) {
		const text = block.serialize(node);
		if (text) return text;
	}
	return null;
}

const serializeParagraph = (node: PMNode) => node.textContent;

const serializeHeading = (node: PMNode) => {
	const level = node.attrs.level || 1;
	return `${"#".repeat(level)} ${node.textContent}`
}

const serializeCodeBlock = (node: PMNode) => {
	const content = node.textContent || "";
	return `\`\`\`\n${content}\n\`\`\``;
}

const serializeList = (node: PMNode, ordered: boolean, indentLevel = 0) => {
	const lines: string[] = [];
	const prefixBase = "\t".repeat(indentLevel);
	let counter = 1;

	node.forEach((item) => {
		let text = "";
		let nestedLists: PMNode[] = [];

		item.forEach(child => {
			if (child.type.name === "paragraph" && text === "") {
				text = child.textContent || "";
			} else if (
				child.type.name === "bullet_list" ||
				child.type.name === "ordered_list"
			) {
				nestedLists.push(child);
			}
		});

		const bullet = ordered ? `${counter}. ` : "- ";
		lines.push(prefixBase + bullet + text);
		counter++;

		for (const nested of nestedLists) {
			const nestedOrdered = nested.type.name === "ordered_list";
			const nestedText = serializeList(nested, nestedOrdered, indentLevel + 1);
			if (nestedText.trim().length > 0) {
				lines.push(nestedText);
			}
		}

		// const text = item.textContent || "";
		// const prefix = ordered ? `${counter}. ` : "- ";
		// lines.push(prefix + text);
		// counter++;
	});

	return lines.join("\n");
}

const serializeOrderedList = (node: PMNode) => {
	return serializeList(node, true);
}

const serializeUnorderedList = (node: PMNode) => {
	return serializeList(node, false);
}

const nodeSerializers: Record<string, (node: PMNode) => string> = {
	"paragraph": serializeParagraph,
	"heading": serializeHeading,
	"bullet_list": serializeUnorderedList,
	"ordered_list": serializeOrderedList,
	"code_block": serializeCodeBlock
}

export const serializeZealotScript = (doc: PMNode): string => {
	const lines: string[] = [];
	doc.forEach((node) => {
		const command = serializeCommandBlock(node);
		if (command) {
			lines.push(command);
			return;
		}

		const kind = node.type.name;
		if (kind in nodeSerializers) {
			const serializer = nodeSerializers[kind];
			const text = serializer(node);
			lines.push(text)
		} else {
			// Default
			lines.push(node.textContent || "");
		}
	});

	return lines.join("\n\n");
}
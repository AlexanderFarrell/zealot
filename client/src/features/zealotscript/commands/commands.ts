import {
	cautionAdmonitionBlock,
	dangerAdmonitionBlock,
	exampleAdmonitionBlock,
	faqAdmonitionBlock,
	importantAdmonitionBlock,
	infoAdmonitionBlock,
	noteAdmonitionBlock,
	successAdmonitionBlock,
	tipAdmonitionBlock,
	todoAdmonitionBlock,
	warningAdmonitionBlock
} from "./admonition";
import TableInfo from "./table";
import type { ZCommandBlockInfo } from "./types";

export const ZCommandBlocks: ZCommandBlockInfo[] = [
	TableInfo,
	noteAdmonitionBlock,
	warningAdmonitionBlock,
	dangerAdmonitionBlock,
	tipAdmonitionBlock,
	infoAdmonitionBlock,
	successAdmonitionBlock,
	importantAdmonitionBlock,
	cautionAdmonitionBlock,
	exampleAdmonitionBlock,
	faqAdmonitionBlock,
	todoAdmonitionBlock
]

export const getCommandBlock = (name: string) => {
	return ZCommandBlocks.find(block => block.name.toLowerCase() === name.toLowerCase());
}

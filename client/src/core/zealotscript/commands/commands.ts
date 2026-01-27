import { dangerAdmonitionBlock, infoAdmonitionBlock, noteAdmonitionBlock, tipAdmonitionBlock, warningAdmonitionBlock } from "./admonition";
import TableInfo from "./table";
import type { ZCommandBlockInfo } from "./types";

export const ZCommandBlocks: ZCommandBlockInfo[] = [
	TableInfo,
	noteAdmonitionBlock,
	warningAdmonitionBlock,
	dangerAdmonitionBlock,
	tipAdmonitionBlock,
	infoAdmonitionBlock,
]

export const getCommandBlock = (name: string) => {
	return ZCommandBlocks.find(block => block.name.toLowerCase() === name.toLowerCase());
}
import type { EditorState, Transaction } from "prosemirror-state";
import { extractYouTubeVideoId } from "./parse/parse_youtube";

export { extractYouTubeVideoId };

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

export const insertTable: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const tableCell = schema.nodes["table_cell"];
	const tableHeader = schema.nodes["table_header"];
	const tableRow = schema.nodes["table_row"];
	const table = schema.nodes["table"];
	if (!paragraph || !tableCell || !tableHeader || !tableRow || !table) return false;

	const makeCell = (header: boolean, text: string) => {
		const p = paragraph.createAndFill({}, schema.text(text))!;
		return header
			? tableHeader.createAndFill({}, [p])!
			: tableCell.createAndFill({}, [p])!;
	};

	const headerRow = tableRow.createAndFill({}, [makeCell(true, "Column 1"), makeCell(true, "Column 2")])!;
	const dataRow = tableRow.createAndFill({}, [makeCell(false, ""), makeCell(false, "")])!;
	const node = table.createAndFill({}, [headerRow, dataRow])!;

	const tr = state.tr.replaceSelectionWith(node).scrollIntoView();
	if (dispatch) dispatch(tr);
	return true;
};

export const insertYoutubeEmbed = (videoId: string): PMCommand => (state, dispatch) => {
	const youtubeEmbed = state.schema.nodes["youtube_embed"];
	if (!youtubeEmbed) return false;
	const node = youtubeEmbed.create({ videoId });
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertAdmonition = (kind: string): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const admonition = schema.nodes["admonition"];
	if (!paragraph || !admonition) return false;

	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = admonition.createAndFill({ kind }, [p])!;
	const tr = state.tr.replaceSelectionWith(node).scrollIntoView();
	if (dispatch) dispatch(tr);
	return true;
};

export const insertMermaidBlock: PMCommand = (state, dispatch) => {
	const codeBlock = state.schema.nodes["code_block"];
	if (!codeBlock) return false;
	const node = codeBlock.create({ language: "mermaid" });
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertDetails = (summary = "Details"): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const details = schema.nodes["details"];
	if (!paragraph || !details) return false;
	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = details.createAndFill({ summary }, [p]);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertSpoiler: PMCommand = (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const spoiler = schema.nodes["spoiler"];
	if (!paragraph || !spoiler) return false;
	const p = paragraph.createAndFill({}, schema.text("..."))!;
	const node = spoiler.createAndFill({}, [p]);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertColumns = (numCols: 2 | 3 | 4 = 2): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const column = schema.nodes["column"];
	const columns = schema.nodes["columns"];
	if (!paragraph || !column || !columns) return false;
	const cols = Array.from({ length: numCols }, () => column.createAndFill({}, [paragraph.create()])!);
	const node = columns.createAndFill({}, cols);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

export const insertTabs = (titles = ["Tab 1", "Tab 2"]): PMCommand => (state, dispatch) => {
	const { schema } = state;
	const paragraph = schema.nodes["paragraph"];
	const tab = schema.nodes["tab"];
	const tabs = schema.nodes["tabs"];
	if (!paragraph || !tab || !tabs) return false;
	const tabNodes = titles.map((title) => tab.createAndFill({ title }, [paragraph.create()])!);
	const node = tabs.createAndFill({}, tabNodes);
	if (!node) return false;
	if (dispatch) dispatch(state.tr.replaceSelectionWith(node).scrollIntoView());
	return true;
};

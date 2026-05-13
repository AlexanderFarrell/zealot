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

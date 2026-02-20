import type { EditorState, Transaction } from "prosemirror-state";

export const insertTable = (state: EditorState, dispatch?: (tr: Transaction) => void) => {
	const {schema} = state;
	const row = schema.nodes.table_row.createAndFill({}, [
		schema.nodes.table_cell.createAndFill({}, schema.nodes.paragraph.createAndFill({}, schema.text("A"))!)!,
		schema.nodes.table_cell.createAndFill({}, schema.nodes.paragraph.createAndFill({}, schema.text("B"))!)!,
	])!;
	const table = schema.nodes.table.createAndFill({}, [row])!;
	const tr = state.tr.replaceSelectionWith(table).scrollIntoView();
	if (dispatch) dispatch(tr);
	return true;
}

export const insertAdmonition = (kind: string) => {
	return (state: EditorState, dispatch?: (tr: Transaction) => void) => {
		const {schema} = state;
		const paragraph = schema.nodes.paragraph.createAndFill({}, schema.text("..."))!;
		const node = schema.nodes.admonition.createAndFill({kind}, paragraph ? [paragraph] : [])!;
		const tr = state.tr.replaceSelectionWith(node).scrollIntoView();
		if (dispatch) dispatch(tr);
		return true;
	}
}
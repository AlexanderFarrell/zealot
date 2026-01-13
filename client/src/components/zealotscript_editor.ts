import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { exampleSetup } from "prosemirror-example-setup";
import ZealotSchema from "../core/zealotscript/schema";
import { parseZealotScript } from "../core/zealotscript/parser";
import { serializeZealotScript } from "../core/zealotscript/serializer";

export type ZealotEditorOptions = {
	content?: string;
	onUpdate?: (value: string, view: EditorView) => void;
	debounceMs?: number;
	handleTab?: boolean;
}

export const createZealotEditorState = (content: string) => {
	return EditorState.create({
		schema: ZealotSchema,
		doc: parseZealotScript(ZealotSchema, content),
		plugins: exampleSetup({ schema: ZealotSchema })
	})
}

export const createZealotEditorView = (
	host: HTMLElement,
	options: ZealotEditorOptions = {}
) => {
	const initial = options.content ?? "";
	const state = createZealotEditorState(initial);
	let saveTimer: number | null = null;
	let lastValue = initial;

	const view = new EditorView(host, {
		state,
		handleKeyDown: (view, event) => {
			if (!options.handleTab) return false;
			if (event.key !== "tab") return false;
			event.preventDefault();
			view.dispatch(view.state.tr.insertText("\t"));
			return true;
		},
		dispatchTransaction: (tr) => {
			const nextState = view.state.apply(tr);
			view.updateState(nextState);

			if (!tr.docChanged || !options.onUpdate) return;
			if (saveTimer !== null) window.clearTimeout(saveTimer);

			const delay = options.debounceMs ?? 500;
			saveTimer = window.setTimeout(() => {
				const value = serializeZealotScript(view.state.doc);
				if (value === lastValue) return;
				lastValue = value;
				options.onUpdate?.(value, view);
			}, delay);
		}
	});

	view.dom.setAttribute("name", "editor_prose");
	return view;
}

export default createZealotEditorView;
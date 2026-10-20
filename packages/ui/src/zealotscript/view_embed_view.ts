import type { Node as PMNode } from "prosemirror-model";
import type { EditorView, NodeView } from "prosemirror-view";
import { getNavigator } from "@websoil/engine";
import { ItemAPI } from "@zealot/api/src/item";
import { ViewAPI, ViewError } from "@zealot/api/src/view";
import type { ViewDisplay } from "@zealot/domain/src/saved_view";
import { ViewResultBox } from "../views/view_result_box";

const itemApi = new ItemAPI("/api");
const viewApi = new ViewAPI("/api");

const SAVED_VIEW_TYPE = "Saved View";

/** Runs the Saved View titled `title`; throws a ViewError the box shows in place. */
export const runViewByTitle = async (title: string, page: { limit: number; offset: number }) => {
	let item;
	try {
		item = await itemApi.GetByTitle(title);
	} catch (error) {
		const status = (error as Error & { response?: Response }).response?.status;
		if (status === 404) throw new ViewError("not_found", `No item titled "${title}"`);
		throw error;
	}
	if (!item.Types.some((t) => t.Name === SAVED_VIEW_TYPE)) {
		throw new ViewError("not_found", "Not a Saved View", item.ItemID);
	}
	return viewApi.Run(item.ItemID, page);
};

/**
 * `![[Title]]` on its own line: the live result of a Saved View, in any container. Editable
 * documents get the cards/table toggle, which writes the mode back into the line; read-only
 * ones (comments) show the result without it.
 */
export class ViewEmbedView implements NodeView {
	dom: HTMLElement;
	private _box = new ViewResultBox();
	private _node: PMNode;

	constructor(
		node: PMNode,
		private readonly _view: EditorView | null,
		private readonly _getPos: (() => number | undefined) | null,
	) {
		this._node = node;
		this.dom = document.createElement("div");
		this.dom.className = "zealot-view-embed";
		this.dom.contentEditable = "false";
		this.dom.appendChild(this._box);
		const title = node.attrs.title as string;
		this._box.init({
			title,
			display: this._display(node),
			load: (page) => runViewByTitle(title, page),
			onMore: () => {
				const id = this._box.viewId;
				if (id !== undefined) getNavigator().openItemById(id);
			},
			...(this._view && this._getPos ? { onDisplayChange: (d: ViewDisplay) => this._setDisplay(d) } : {}),
		});
	}

	private _display(node: PMNode): ViewDisplay {
		return node.attrs.display === "table" ? "table" : "cards";
	}

	private _setDisplay(display: ViewDisplay): void {
		const pos = this._getPos?.();
		if (!this._view || pos === undefined) return;
		this._view.dispatch(
			this._view.state.tr.setNodeMarkup(pos, undefined, { ...this._node.attrs, display }),
		);
	}

	update(node: PMNode): boolean {
		if (node.type !== this._node.type || node.attrs.title !== this._node.attrs.title) return false;
		this._node = node;
		this._box.setDisplay(this._display(node));
		return true;
	}

	stopEvent(): boolean { return true; }
	ignoreMutation(): boolean { return true; }
}

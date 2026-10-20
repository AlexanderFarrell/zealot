import type { Node as PMNode } from "prosemirror-model";
import type { EditorView, NodeView } from "prosemirror-view";
import { ViewAPI, ViewError } from "@zealot/api/src/view";
import type { ViewDisplay } from "@zealot/domain/src/saved_view";
import { ViewBuilder } from "../views/view_builder";
import { ViewResultBox } from "../views/view_result_box";

const viewApi = new ViewAPI("/api");

export interface SavedViewHost {
	viewId: number;
	title: string;
	/** Open the builder as soon as the block renders (a view just created). */
	startEditing?: boolean;
}

const pendingEdits = new Set<number>();

/** A view just created opens in Edit view once, the next time its page renders. */
export const markStartEditing = (itemId: number): void => { pendingEdits.add(itemId); };
export const consumeStartEditing = (itemId: number): boolean => pendingEdits.delete(itemId);

/** True for a fenced block whose info string starts with `zealot-view`, as the server reads it. */
export const isViewFence = (node: PMNode): boolean =>
	node.type.name === "code_block" && ((node.attrs.language as string) ?? "").trim().split(/\s+/)[0] === "zealot-view";

/** The server accepts one definition per item, so only the first block is a live result. */
export const isFirstViewBlock = (view: EditorView, getPos: () => number | undefined): boolean => {
	const pos = getPos();
	if (pos === undefined) return false;
	let first: number | null = null;
	view.state.doc.descendants((node, nodePos) => {
		if (first !== null) return false;
		if (isViewFence(node)) { first = nodePos; return false; }
		return true;
	});
	return first === pos;
};

/** Reads the definition JSON of a block; throws a ViewError the box shows in place. */
export const readDefinition = (text: string): Record<string, unknown> => {
	try {
		const value: unknown = JSON.parse(text);
		if (value === null || typeof value !== "object" || Array.isArray(value)) {
			throw new Error("a definition is a JSON object");
		}
		return value as Record<string, unknown>;
	} catch (error) {
		throw new ViewError("invalid", `Invalid definition: ${(error as Error).message}`);
	}
};

/**
 * The live result in place of a Saved View's `zealot-view` block. The block stays the stored
 * form: the box runs whatever JSON the block holds, and every change goes back as an editor
 * transaction so the item saves through its normal path.
 */
export class SavedViewBlockView implements NodeView {
	dom: HTMLElement;
	private _box = new ViewResultBox();
	private _builder: ViewBuilder | null = null;
	private _node: PMNode;
	private _definition: Record<string, unknown> | null = null;

	constructor(
		node: PMNode,
		private readonly _view: EditorView,
		private readonly _getPos: () => number | undefined,
		private readonly _host: SavedViewHost,
	) {
		this._node = node;
		this.dom = document.createElement("div");
		this.dom.className = "zealot-view-block";
		this.dom.contentEditable = "false";
		this.dom.appendChild(this._box);
		this._start();
		if (_host.startEditing) queueMicrotask(() => this._openBuilder());
	}

	private _display(): ViewDisplay {
		return this._definition?.display === "table" ? "table" : "cards";
	}

	private _start(): void {
		try {
			this._definition = readDefinition(this._node.textContent);
		} catch {
			this._definition = null;
		}
		this._box.init({
			title: this._host.title,
			viewId: this._host.viewId,
			display: this._display(),
			paged: true,
			load: (page) => viewApi.RunDefinition(readDefinition(this._node.textContent), page),
			onDisplayChange: (display) => this._setDisplay(display),
			onEdit: () => this._openBuilder(),
			onFix: () => this._openBuilder(),
		});
	}

	/** Swaps the result for the builder; Done writes the block through the editor, Cancel restores it. */
	private _openBuilder(): void {
		if (this._builder) return;
		const builder = new ViewBuilder();
		this._builder = builder;
		this.dom.replaceChildren(builder);
		builder.init({
			text: this._node.textContent,
			title: this._host.title,
			onCancel: () => this._closeBuilder(),
			onDone: (text) => {
				this._closeBuilder();
				if (text !== null) this._writeText(text);
			},
		});
	}

	private _closeBuilder(): void {
		this._builder = null;
		this.dom.replaceChildren(this._box);
		void this._box.refresh();
	}

	/** Replaces the block's text, not the document, so other boxes do not refetch. */
	private _writeText(text: string): void {
		const pos = this._getPos();
		if (pos === undefined) return;
		const { schema } = this._view.state;
		this._view.dispatch(
			this._view.state.tr.replaceWith(pos + 1, pos + 1 + this._node.content.size, schema.text(text)),
		);
	}

	/** Writes `display` into the definition; the block's text is replaced, not the document. */
	private _setDisplay(display: ViewDisplay): void {
		const pos = this._getPos();
		if (pos === undefined || !this._definition) return;
		this._writeText(JSON.stringify({ ...this._definition, display }, null, 2));
	}

	/** Everything in the definition except `display`, which only changes how it is shown. */
	private _queryKey(): string {
		const { display: _display, ...rest } = this._definition ?? {};
		return JSON.stringify(rest);
	}

	update(node: PMNode): boolean {
		if (node.type !== this._node.type || !isViewFence(node)) return false;
		if (node.textContent === this._node.textContent) {
			this._node = node;
			return true;
		}
		this._node = node;
		const before = this._queryKey();
		try { this._definition = readDefinition(node.textContent); } catch { this._definition = null; }
		this._box.setDisplay(this._display());
		// A display write-back only changes the mode; anything else reruns the query.
		if (this._queryKey() !== before) void this._box.refresh();
		return true;
	}

	stopEvent(): boolean { return true; }
	ignoreMutation(): boolean { return true; }
}

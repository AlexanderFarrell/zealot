import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { getNavigator } from "@websoil/engine";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { YoutubeEmbedView } from "./zealotscript_editor";

export class ZealotScriptView extends HTMLElement {
	private _view: EditorView | null = null;
	private _content = "";

	connectedCallback(): void {
		if (this._view) return;

		const container = document.createElement("div");
		container.className = "zealotscript-view-container";
		this.appendChild(container);

		const state = EditorState.create({
			schema: ZealotSchema,
			doc: parseZealotScript(ZealotSchema, this._content),
		});

		this._view = new EditorView(container, {
			state,
			editable: () => false,
			nodeViews: {
				youtube_embed: (node) => new YoutubeEmbedView(node),
			},
			handleClick: (_view, _pos, event) => {
				const anchor = (event.target as HTMLElement)?.closest("a[href]") as HTMLAnchorElement | null;
				if (!anchor) return false;
				const href = anchor.getAttribute("href") ?? "";
				if (!href) return false;
				event.preventDefault();
				event.stopPropagation();
				if (href.startsWith("zealot://item/")) {
					getNavigator().openItem(decodeURIComponent(href.slice("zealot://item/".length)));
				} else if (href.startsWith("zealot://type/")) {
					getNavigator().openType(decodeURIComponent(href.slice("zealot://type/".length)));
				} else {
					window.open(href, "_blank", "noopener,noreferrer");
				}
				return true;
			},
		});
	}

	disconnectedCallback(): void {
		this._view?.destroy();
		this._view = null;
	}

	get content(): string {
		return this._content;
	}

	set content(value: string) {
		this._content = value;
		if (!this._view) return;
		const doc = parseZealotScript(ZealotSchema, value);
		this._view.dispatch(
			this._view.state.tr.replaceWith(0, this._view.state.doc.content.size, doc.content),
		);
	}
}

if (!customElements.get("zealotscript-view")) {
	customElements.define("zealotscript-view", ZealotScriptView);
}

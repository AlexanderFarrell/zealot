import "katex/dist/katex.min.css";
import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { Node as PMNode } from "prosemirror-model";
import type katex from "katex";
import type MermaidType from "mermaid";
import { getNavigator } from "@websoil/engine";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { YoutubeEmbedView } from "./zealotscript_editor";

let katexLib: typeof katex | null = null;
const loadKatex = async (): Promise<typeof katex> => {
	if (katexLib) return katexLib;
	const mod = await import("katex");
	katexLib = mod.default;
	return katexLib;
};

let mermaidLib: typeof MermaidType | null = null;
const loadMermaid = async (): Promise<typeof MermaidType> => {
	if (mermaidLib) return mermaidLib;
	const mod = await import("mermaid");
	mermaidLib = mod.default;
	mermaidLib.initialize({ startOnLoad: false, theme: "neutral" });
	return mermaidLib;
};

let mermaidCounter = 0;

const renderMermaidInContainer = async (container: HTMLElement): Promise<void> => {
	const blocks = Array.from(container.querySelectorAll<HTMLElement>("pre[data-language='mermaid']"));
	if (blocks.length === 0) return;
	const mermaid = await loadMermaid();
	for (const pre of blocks) {
		const source = pre.textContent ?? "";
		const id = `zealot-mermaid-${++mermaidCounter}`;
		const wrapper = document.createElement("div");
		wrapper.className = "zealot-mermaid";
		try {
			const { svg } = await mermaid.render(id, source);
			wrapper.innerHTML = svg;
		} catch (err) {
			wrapper.className = "zealot-mermaid-error";
			const banner = document.createElement("div");
			banner.className = "zealot-mermaid-error-banner";
			banner.textContent = err instanceof Error ? err.message : String(err);
			const raw = document.createElement("pre");
			raw.textContent = source;
			wrapper.appendChild(banner);
			wrapper.appendChild(raw);
		}
		pre.replaceWith(wrapper);
	}
};

const renderMathInContainer = async (container: HTMLElement): Promise<void> => {
	const lib = await loadKatex();

	for (const el of Array.from(container.querySelectorAll<HTMLElement>("span[data-math-inline]"))) {
		const src = el.getAttribute("data-math-inline") ?? "";
		try {
			el.innerHTML = lib.renderToString(src, { displayMode: false, throwOnError: true });
		} catch {
			el.innerHTML = "";
			const err = document.createElement("span");
			err.className = "zealot-math-error";
			err.textContent = src;
			el.appendChild(err);
		}
	}

	for (const el of Array.from(container.querySelectorAll<HTMLElement>("div[data-math-block]"))) {
		const src = el.getAttribute("data-math-block") ?? "";
		try {
			el.innerHTML = lib.renderToString(src, { displayMode: true, throwOnError: true });
		} catch {
			el.innerHTML = "";
			const err = document.createElement("span");
			err.className = "zealot-math-error";
			err.textContent = src;
			el.appendChild(err);
		}
	}
};

export class TabsView {
	dom: HTMLElement;
	contentDOM: HTMLElement;
	private _tabBar: HTMLElement;
	private _activeIndex = 0;
	private _observer: MutationObserver;
	private _node: PMNode;

	constructor(node: PMNode) {
		this._node = node;

		const wrapper = document.createElement("div");
		wrapper.className = "zealot-tabs";

		const tabBar = document.createElement("div");
		tabBar.className = "zealot-tabs-bar";
		wrapper.appendChild(tabBar);

		const panels = document.createElement("div");
		panels.className = "zealot-tabs-panels";
		wrapper.appendChild(panels);

		this.dom = wrapper;
		this.contentDOM = panels;
		this._tabBar = tabBar;

		this._buildTabBar();

		this._observer = new MutationObserver(() => this._applyActiveTab());
		this._observer.observe(panels, { childList: true });
	}

	private _buildTabBar(): void {
		this._tabBar.innerHTML = "";
		this._node.forEach((tabNode, _, index) => {
			const title = (tabNode.attrs.title as string) || `Tab ${index + 1}`;
			const btn = document.createElement("button");
			btn.type = "button";
			btn.className = "zealot-tab-btn";
			if (index === this._activeIndex) btn.classList.add("zealot-tab-btn--active");
			btn.textContent = title;
			btn.addEventListener("mousedown", (e) => {
				e.preventDefault();
				this._activeIndex = index;
				this._buildTabBar();
				this._applyActiveTab();
			});
			this._tabBar.appendChild(btn);
		});
	}

	private _applyActiveTab(): void {
		const panels = Array.from(this.contentDOM.children) as HTMLElement[];
		panels.forEach((panel, i) => {
			panel.style.display = i === this._activeIndex ? "" : "none";
		});
	}

	update(node: PMNode): boolean {
		if (node.type !== this._node.type) return false;
		this._node = node;
		this._activeIndex = Math.min(this._activeIndex, Math.max(0, node.childCount - 1));
		this._buildTabBar();
		this._applyActiveTab();
		return true;
	}

	destroy(): void {
		this._observer.disconnect();
	}
}

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
				tabs: (node) => new TabsView(node),
			},
			dispatchTransaction: (tr) => {
				if (!this._view) return;
				this._view.updateState(this._view.state.apply(tr));
				if (tr.docChanged) {
					renderMathInContainer(container);
					renderMermaidInContainer(container);
				}
			},
			handleClick: (view, _pos, event) => {
				if (event.button !== 0) return false;
				const target = event.target as HTMLElement;

				if (target instanceof HTMLInputElement && target.type === "checkbox" && target.closest("li[data-checked]")) {
					event.preventDefault();
					const li = target.closest("li[data-checked]");
					if (!li) return false;
					let nodePos: number | null = null;
					view.state.doc.nodesBetween(0, view.state.doc.content.size, (node, p) => {
						if (node.type === view.state.schema.nodes["list_item"] && node.attrs.checked !== null) {
							const domNode = view.nodeDOM(p);
							if (domNode === li || (domNode as HTMLElement)?.contains(li)) {
								nodePos = p;
							}
						}
					});
					if (nodePos !== null) {
						const node = view.state.doc.nodeAt(nodePos);
						if (node) {
							const tr = view.state.tr.setNodeMarkup(nodePos, undefined, {
								...node.attrs,
								checked: !node.attrs.checked,
							});
							view.dispatch(tr);
							const updated = serializeZealotScript(view.state.doc);
							this.dispatchEvent(new CustomEvent("item:content-changed", { detail: updated, bubbles: true }));
						}
					}
					return true;
				}

				const dateRef = target.closest<HTMLElement>("span[data-date-ref]");
				if (dateRef) {
					const raw = dateRef.getAttribute("data-date-ref") ?? "";
					const kind = dateRef.getAttribute("data-date-kind") ?? "day";
					event.preventDefault();
					event.stopPropagation();
					if (kind === "day") getNavigator().openPlanner("daily", raw);
					else if (kind === "week") getNavigator().openPlanner("weekly", raw);
					else getNavigator().openPlanner("annual", raw);
					return true;
				}

				const anchor = target?.closest("a[href]") as HTMLAnchorElement | null;
				if (!anchor) return false;
				const href = anchor.getAttribute("href") ?? "";
				if (!href) return false;
				event.preventDefault();
				event.stopPropagation();
				if (href.startsWith("zealot://item/")) {
					const raw = decodeURIComponent(href.slice("zealot://item/".length));
					const hashIdx = raw.indexOf("#");
					const itemId = hashIdx >= 0 ? raw.slice(0, hashIdx) : raw;
					const anchorText = hashIdx >= 0 ? raw.slice(hashIdx + 1).trim() : "";
					getNavigator().openItem(itemId);
					if (anchorText.length > 0) {
						requestAnimationFrame(() => {
							const headings = Array.from(document.querySelectorAll<HTMLElement>("h1,h2,h3,h4,h5,h6"));
							const target = headings.find(
								(h) => h.textContent?.trim().toLowerCase() === anchorText.toLowerCase()
							);
							target?.scrollIntoView({ behavior: "smooth", block: "start" });
						});
					}
				} else if (href.startsWith("zealot://type/")) {
					getNavigator().openType(decodeURIComponent(href.slice("zealot://type/".length)));
				} else {
					window.open(href, "_blank", "noopener,noreferrer");
				}
				return true;
			},
		});

		renderMathInContainer(container);
		renderMermaidInContainer(container);
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

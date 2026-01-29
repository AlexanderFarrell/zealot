import type { Item } from "../api/item";
import { GenericSearch } from "./common/generic_search";

class ItemSearchInline extends GenericSearch<Item> {
	private selected: Item | null = null;
	private input: HTMLInputElement | null = null;
	private resultsView: HTMLDivElement | null = null;

	async render() {
		await super.render();
		this.input = this.querySelector('[name="search"]') as HTMLInputElement;
		this.resultsView = this.querySelector('[name="generic_results"]') as HTMLDivElement;
		if (this.input) {
			this.input.addEventListener("keydown", (e: KeyboardEvent) => {
				if (e.key === "Enter") {
					e.preventDefault();
				}
			});
		}
	}

	setSelected(item: Item) {
		this.selected = item;
		if (this.input) {
			const icon = item.attributes?.["Icon"] || "";
			this.input.value = `${icon} ${item.title}`.trim();
		}
		if (this.resultsView) {
			this.resultsView.innerHTML = "";
		}
		this.dispatchEvent(new CustomEvent("item-selected", {detail: {item}}));
	}

	clear() {
		this.selected = null;
		if (this.input) {
			this.input.value = "";
		}
		if (this.resultsView) {
			this.resultsView.innerHTML = "&nbsp;";
		}
	}

	get value(): Item | null {
		return this.selected;
	}
}

customElements.define("item-search-inline", ItemSearchInline);

export default ItemSearchInline;

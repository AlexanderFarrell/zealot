import API from "../../api/api";
import { type Item } from "../../api/item";
import { BaseElementEmpty } from "../../shared/base_element";
import { GenericSearch } from "../../shared/generic_search";
import { router } from "../router/router";

class ItemSearch extends GenericSearch<Item>{}

export class ItemSearchModal extends BaseElementEmpty {
	render() {
		this.classList.add('modal_background')

		this.innerHTML = `
		<div class="inner_window">
			<item-search></item-search>
		</div>
		`

		let inner_window = this.querySelector('.inner_window')! as HTMLDivElement;
		
		this.addEventListener('keydown', (e: KeyboardEvent) => {
			if (e.key == "Escape") {
				this.remove();
			}
		})

		inner_window.addEventListener('click', (e: MouseEvent) => {
			e.stopPropagation();
		})
		this.addEventListener('click', (e: MouseEvent) => {
			this.remove();
		})

		let search_view = this.querySelector('item-search')! as ItemSearch;
		search_view.init({
			on_search: async (term: string) => {
				return await API.item.search(term) as Array<Item>;
			},
			on_match_text: (item: Item) => item.title,
			on_select: (item: Item) => {
				router.navigate(`/item_id/${item.item_id}`)
				this.remove();
			},
			on_make_view: (item: Item) => {
				let div = document.createElement('div');
				div.innerText = `${item.attributes!['Icon'] || ""} ${item.title}`;
				return div;
			}
		})
		search_view.focus();
	}
}

customElements.define('item-search', ItemSearch);
customElements.define('item-search-modal', ItemSearchModal);

export default ItemSearchModal;

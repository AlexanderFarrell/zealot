import API from "../../api/api";
import type { ItemType } from "../../api/item_type";
import BaseElement from "../../shared/base_element";
import ItemTable from "../item/item_table";
import ItemAnalysis from "../item/item_analysis";

class TypeScreen extends BaseElement<ItemType> {
	async render() {
		this.classList.add('center');
		let type = this.data!;

		this.innerHTML = `
			<h1>${type.name} Items</h1>
			<item-analysis></item-analysis>
			<item-table></item-table>
			`;
		let table = this.querySelector('item-table')! as ItemTable;
		let analysis = this.querySelector('item-analysis')! as ItemAnalysis;

		let items = await API.item.get_by_type(type.name);
		analysis.init(items);
		table.init({
			items: items,
			columns: type.required_attribute_keys!
		})
		table.item_type = type.name;
	}
}

customElements.define('type-screen', TypeScreen);

export default TypeScreen;
import type { Item } from "../api/item";
import BaseElement from "./common/base_element";

function compute_total_score(items: Item[]): number {
	let score = 0;
	items.forEach(item => {
		let priority_score = (item.attributes!['Priority'] | 0) * 100;
		score += priority_score;
	})
	return score;
}

function compute_completed_score(items: Item[]): number {
	let score = 0;
	items.forEach(item => {
		if (item.attributes!['Status'] == null) {
			return;
		}
		let priority_score = (item.attributes!['Priority'] | 0) * 100;
		if (item.attributes!['Status'] == 'Complete' || item.attributes!['Status'] == 'Rejected') {
			score += priority_score;
		}
	})
	return score;
}

class ItemAnalysis extends BaseElement<Item[]> {
	render() {
		this.style.display = 'block';
		let total_score = compute_total_score(this.data!);
		let completed_score = compute_completed_score(this.data!);


		this.innerText = `Score: ${completed_score}/${total_score}`
	}
}

customElements.define('item-analysis', ItemAnalysis);

export default ItemAnalysis;
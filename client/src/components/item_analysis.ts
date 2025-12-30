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

interface StatsBreakdown {
	attribute: string,
	totals: Map<string, number>,
	percentages: Map<string, number>
	item_count: number
}

function compute_stats_breakdown(items: Item[], attribute: string): StatsBreakdown {
	let breakdown = {
		attribute: attribute,
		totals: new Map<string, number>(),
		percentages: new Map<string, number>(),
		item_count: items.length
	};

	// Compute totals
	items.forEach(item => {
		let val = `${item.attributes![attribute]}`
		if (!breakdown.totals.has(val)) {
			breakdown.totals.set(val, 0);
		}
		breakdown.totals.set(val, breakdown.totals.get(val)! + 1);
	})

	// Compute percentages 
	breakdown.totals.forEach((total, key) => {
		breakdown.percentages.set(key, total / breakdown.item_count)
	})

	return breakdown;
}

function make_breakdown_view(breakdown: StatsBreakdown) {
	if (breakdown.totals.size == 1) {
		return ``
	}

	let attrHTML = ''
	breakdown.totals.forEach((val, key) => {
		attrHTML += `(${key} - ${toPercent(breakdown.percentages.get(key)!)})`
		// attrHTML += ` (${key}: Total: ${val} - Percent: ${toPercent(breakdown.percentages.get(key)!)}) `
	})

	return `<div>
		${breakdown.attribute}: 
		${attrHTML}
	</div>`
}

function toPercent(n: number) {
	return new Intl.NumberFormat(
		Intl.DateTimeFormat().resolvedOptions().locale,
		{style: "percent", maximumFractionDigits: 2}
	).format(n)
}

class ItemAnalysis extends BaseElement<Item[]> {
	render() {
		this.style.display = 'block';
		let total_score = compute_total_score(this.data!);
		if (total_score == 0) {
			return;
		}
		let completed_score = compute_completed_score(this.data!);
		let stats_breakdown = compute_stats_breakdown(this.data!, "Status")
		// let priority_breakdown = compute_stats_breakdown(this.data!, "Priority")

		this.innerHTML = `Score: ${completed_score} / ${total_score}
		`
	}
}

customElements.define('item-analysis', ItemAnalysis);

export default ItemAnalysis;
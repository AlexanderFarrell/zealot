import type { Item } from "../api/item";
import Analysis from "../core/analysis";
import BaseElement from "./common/base_element";
import type { PieChartView } from "./graphs";

class ItemAnalysis extends BaseElement<Item[]> {
	render() {
		let items = this.data!;
		this.style.display = 'block';
		let total_score = Analysis.compute_total_score(items);
		if (total_score == 0) {
			return;
		}
		let completed_score = Analysis.compute_completed_score(items);

		this.innerHTML = `<div name="total">Score: ${completed_score} / ${total_score}</div>
		<div name="breakdown"></div>`

		let are_pie_charts_visible = false;
		(this.querySelector('[name="total"]')! as HTMLDivElement).addEventListener('click', () => {
			are_pie_charts_visible = !are_pie_charts_visible;
			let breakdown = this.querySelector('[name="breakdown"]')! as HTMLDivElement;
			if (are_pie_charts_visible) {
				breakdown.innerHTML = `
				<div class="row">
					<pie-chart name="status"></pie-chart>
					<pie-chart name="priority"></pie-chart>
				</div>
				<div class="row">
					<pie-chart name="action-points"></pie-chart>
					<pie-chart name="parent"></pie-chart>
				</div>
				`

				let status_pie_chart = this.querySelector('[name="status"]')! as PieChartView;
				let priority_pie_chart = this.querySelector('[name="priority"]')! as PieChartView;
				let action_points_pie_chart = this.querySelector('[name="action-points"]')! as PieChartView;
				let parent_pie_chart = this.querySelector('[name="parent"]')! as PieChartView;

				status_pie_chart.init({
					caption: "Status",
					items: Analysis.group_by_sum(items, "Status")
				})
				priority_pie_chart.init({
					caption: "Priority",
					items: Analysis.group_by_sum(items, "Priority")
				})
				action_points_pie_chart.init({
					caption: "Action Points",
					items: Analysis.group_by_sum(items, "AP")
				})
				parent_pie_chart.init({
					caption: "Parents",
					items: Analysis.group_by_sum(items, "Parent")
				})
			} else {
				breakdown.innerHTML = "";
			}
		})

	}
}

customElements.define('item-analysis', ItemAnalysis);

export default ItemAnalysis;
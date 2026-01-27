import { DateTime } from "luxon";
import { BaseElementEmpty } from "../components/common/base_element";
import { type AttributeFilter, type Item } from "../api/item";
import API from "../api/api";
import { LineGraphView, PieChartView } from "../components/graphs";
import Analysis from "../core/analysis";

class AnalysisScreen extends BaseElementEmpty {
    async render() {
        let days = 30;

        // Get last 30 days
        const items = await Analysis.get_items_days(days);

        const days_x = []
        const completed_y = []
        const score_y = []
        for (let i = 0; i < days; i++) {
            let day = DateTime.now().plus({days: i - days}).toISODate();
            days_x.push(day)
            let completed = 0
            let score = 0

            items.forEach(item => {
                if (item.attributes!["Date"]?.substring(0, 10) != day) {
                    return;
                }
                if (item.attributes!['Status'] == "Complete" || 
                    item.attributes!['Status'] == "Cancelled") {
                        completed++;

                        let priority: number = item.attributes!['Priority'] || 1;
                        let ap: number = item.attributes!['AP'] | 1;
                        score += 100 * priority * ap;
                    }
            })
            completed_y.push(completed)
            score_y.push(score)
        }

        this.classList.add('center')
        this.innerHTML = `<h1>Analysis</h1>
        <h2>Last ${days} Days</h2>
        <div>Items: ${items.length}</div>
        <div class="row">
            <pie-chart name="status"></pie-chart>
            <pie-chart name="priority"></pie-chart>
            <pie-chart name="action-points"></pie-chart>
        </div>
        <line-graph name="score"></line-graph>
        <line-graph name="completed"></line-graph>
        <div>
            <pie-chart name="parent"></pie-chart>
        </div>
        `

        let status_pie_chart = this.querySelector('[name="status"]')! as PieChartView;
        let priority_pie_chart = this.querySelector('[name="priority"]')! as PieChartView;
        let action_points_pie_chart = this.querySelector('[name="action-points"]')! as PieChartView;
        let parent_pie_chart = this.querySelector('[name="parent"]')! as PieChartView;
        let completed_line = this.querySelector('[name="completed"]')! as LineGraphView;
        let score_line = this.querySelector('[name="score"]')! as LineGraphView;

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

        score_line.init({
            caption: "Scorecard",
            x: days_x,
            y: score_y
        });
        completed_line.init({
            caption: "Completed Goals",
            x: days_x,
            y: completed_y
        })
    }
}

customElements.define('analysis-screen', AnalysisScreen)

export default AnalysisScreen;
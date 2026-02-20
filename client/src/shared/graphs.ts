import BaseElement from "./base_element";
import {Chart, PieController, ArcElement, Tooltip, Legend, LineController, LineElement, LinearScale, CategoryScale, PointElement} from "chart.js";
Chart.register(PieController, ArcElement, Tooltip, Legend, LineController, LineElement, LinearScale, CategoryScale, PointElement);

const border_color = "#3185FC";
const background_color = "#1c4379ff"

const base_html = `
		<div class="caption"></div>
		<div style="width: 100%; height: 280px">
			<canvas></canvas>
		</div>
		`

const pie_palette = [
    "#1B998B", "#E84855", "#F9DC5C", "#3185FC",
    "#6A4C93", "#FF9F1C", "#2EC4B6", "#E71D36",
    "#00A6FB", "#F7A072"
];

export interface LineGraphInfo {
	caption: string, 
	x: string[],
	y: number[]
}

export interface PieChartInfo {
	caption: string,
	items: Record<string, number>
}

export class LineGraphView extends BaseElement<LineGraphInfo> {
	render() {
		this.innerHTML = base_html;
		(this.querySelector('.caption')! as HTMLDivElement).innerText = this.data!.caption;

		const canvas = this.querySelector('canvas')! as HTMLCanvasElement;
		new Chart(canvas, {
			type: "line",
			data: {
				labels: this.data!.x,
				datasets: [
					{
						label: this.data!.caption,
						data: this.data!.y,
						borderColor:  border_color,
						backgroundColor: background_color,
						tension: 0.25,
						fill: true
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { position: "top" },
					title: {display: true, text: this.data!.caption}
				},
				scales: {
					y: {beginAtZero: true, ticks: {precision: 0}}
				}
			}
		})
	}
};

export class PieChartView extends BaseElement<PieChartInfo> {
	render() {
		this.innerHTML = base_html;
		(this.querySelector('.caption')! as HTMLDivElement).innerText = this.data!.caption;
		const canvas = this.querySelector('canvas')! as HTMLCanvasElement;

		const labels = Object.keys(this.data!.items);
		const values = Object.values(this.data!.items);
		const colors = labels.map((_, i) => pie_palette[i % pie_palette.length]);

		new Chart(canvas, {
			type: "pie",
			data: {
				labels,
				datasets: [
					{
						data: values,
						backgroundColor: colors,
						borderColor: "#ffffff",
						borderWidth: 1
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: true,
				plugins: {
					legend: {position: "right"},
					title: {display: true, text: this.data!.caption}
				}
			}
		})
	}
}

customElements.define('line-graph', LineGraphView);
customElements.define('pie-chart', PieChartView);
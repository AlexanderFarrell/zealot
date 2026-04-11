import { DateTime } from 'luxon';
import { BaseElementEmpty, graphs, getNavigator } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { LoadingSpinner } from '../common/loading_spinner';
import { ItemTableView, type ItemTableColumn } from '../views/item_table_view';

const statusTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
    { attributeKey: 'AP', kind: 'attribute', label: 'AP' },
    { attributeKey: 'Date', kind: 'attribute', label: 'Date' },
];

const itemApi = new ItemAPI('/api');

class AnalysisUtils {
    static async getItemsByDays(days: number): Promise<Item[]> {
        const since = DateTime.now().minus({ days: days - 1 }).toISODate()!;
        return itemApi.Filter([{ key: 'Date', op: 'gte', value: since, list_mode: 'any' }]);
    }

    static async getItemsByStatus(status: string): Promise<Item[]> {
        return itemApi.Filter([{ key: 'Status', op: 'eq', value: status, list_mode: 'any' }]);
    }

    static groupBySum(items: Item[], attribute: string): Record<string, number> {
        const categories: Record<string, number> = {};
        items.forEach(item => {
            let val = item.Attributes?.[attribute];
            if (val == null) val = 'None';
            categories[val] = (categories[val] ?? 0) + 1;
        });
        return categories;
    }

    static computeScoreTimeSeries(items: Item[], days: number): { x: string[]; y: number[] } {
        const x: string[] = [];
        const y: number[] = [];
        for (let i = 0; i < days; i++) {
            const day = DateTime.now().plus({ days: i - days }).toISODate()!;
            x.push(day);
            let score = 0;
            items.forEach(item => {
                if (item.Attributes?.['Date']?.substring(0, 10) !== day) return;
                const status = item.Attributes?.['Status'];
                if (status === 'Complete' || status === 'Cancelled') {
                    const priority: number = item.Attributes?.['Priority'] ?? 1;
                    const ap: number = item.Attributes?.['AP'] ?? 1;
                    score += 100 * priority * ap;
                }
            });
            y.push(score);
        }
        return { x, y };
    }

    static computeCompletedTimeSeries(items: Item[], days: number): { x: string[]; y: number[] } {
        const x: string[] = [];
        const y: number[] = [];
        for (let i = 0; i < days; i++) {
            const day = DateTime.now().plus({ days: i - days }).toISODate()!;
            x.push(day);
            let completed = 0;
            items.forEach(item => {
                if (item.Attributes?.['Date']?.substring(0, 10) !== day) return;
                const status = item.Attributes?.['Status'];
                if (status === 'Complete' || status === 'Cancelled') completed++;
            });
            y.push(completed);
        }
        return { x, y };
    }
}

function renderTabBar(active: 'analysis' | 'specify' | 'working'): HTMLElement {
    const bar = document.createElement('div');
    bar.className = 'analysis-tabs';

    const tabs: { label: string; key: 'analysis' | 'specify' | 'working' }[] = [
        { label: 'Analysis', key: 'analysis' },
        { label: 'Specify', key: 'specify' },
        { label: 'Working', key: 'working' },
    ];

    tabs.forEach(tab => {
        const btn = document.createElement('button');
        btn.textContent = tab.label;
        if (tab.key === active) btn.classList.add('is-active');
        btn.addEventListener('click', () => {
            const nav = getNavigator();
            if (tab.key === 'analysis') nav.openAnalysis();
            else if (tab.key === 'specify') nav.openAnalysisSpecify();
            else nav.openAnalysisWorking();
        });
        bar.appendChild(btn);
    });

    return bar;
}

function renderPieRow(items: Item[]): HTMLElement {
    const row = document.createElement('div');
    row.className = 'analysis-charts-row';

    const statusChart = new graphs.PieChartView();
    statusChart.className = 'analysis-chart';
    statusChart.init({ caption: 'Status', items: AnalysisUtils.groupBySum(items, 'Status') });

    const priorityChart = new graphs.PieChartView();
    priorityChart.className = 'analysis-chart';
    priorityChart.init({ caption: 'Priority', items: AnalysisUtils.groupBySum(items, 'Priority') });

    const apChart = new graphs.PieChartView();
    apChart.className = 'analysis-chart';
    apChart.init({ caption: 'Action Points', items: AnalysisUtils.groupBySum(items, 'AP') });

    row.appendChild(statusChart);
    row.appendChild(priorityChart);
    row.appendChild(apChart);
    return row;
}

export class AnalysisScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('analysis'));

        const heading = document.createElement('h1');
        heading.textContent = 'Analysis';
        wrapper.appendChild(heading);

        const subheading = document.createElement('h2');
        subheading.textContent = 'Last 30 Days';
        wrapper.appendChild(subheading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const days = 30;
        const items = await AnalysisUtils.getItemsByDays(days);
        wrapper.removeChild(spinner);

        const count = document.createElement('div');
        count.textContent = `Items: ${items.length}`;
        wrapper.appendChild(count);

        wrapper.appendChild(renderPieRow(items));

        const parentRow = document.createElement('div');
        parentRow.className = 'analysis-charts-row';
        const parentChart = new graphs.PieChartView();
        parentChart.className = 'analysis-chart';
        parentChart.init({ caption: 'Parents', items: AnalysisUtils.groupBySum(items, 'Parent') });
        parentRow.appendChild(parentChart);
        wrapper.appendChild(parentRow);

        const scoreData = AnalysisUtils.computeScoreTimeSeries(items, days);
        const scoreGraph = new graphs.LineGraphView();
        scoreGraph.init({ caption: 'Scorecard', x: scoreData.x, y: scoreData.y });
        wrapper.appendChild(scoreGraph);

        const completedData = AnalysisUtils.computeCompletedTimeSeries(items, days);
        const completedGraph = new graphs.LineGraphView();
        completedGraph.init({ caption: 'Completed Goals', x: completedData.x, y: completedData.y });
        wrapper.appendChild(completedGraph);
    }
}

customElements.define('analysis-screen', AnalysisScreen);

export class SpecifyScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('specify'));

        const heading = document.createElement('h1');
        heading.textContent = 'Specify';
        wrapper.appendChild(heading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const items = await AnalysisUtils.getItemsByStatus('Specify');
        wrapper.removeChild(spinner);

        const table = new ItemTableView();
        table.init({ columns: statusTableColumns, items, emptyMessage: 'No items in Specify.' });
        wrapper.appendChild(table);
    }
}

customElements.define('specify-screen', SpecifyScreen);

export class WorkingScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('working'));

        const heading = document.createElement('h1');
        heading.textContent = 'Working';
        wrapper.appendChild(heading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const items = await AnalysisUtils.getItemsByStatus('Working');
        wrapper.removeChild(spinner);

        const table = new ItemTableView();
        table.init({ columns: statusTableColumns, items, emptyMessage: 'No items in Working.' });
        wrapper.appendChild(table);
    }
}

customElements.define('working-screen', WorkingScreen);

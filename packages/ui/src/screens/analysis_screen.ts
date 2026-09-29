import { DateTime } from 'luxon';
import { BaseElementEmpty, graphs, getNavigator } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { LoadingSpinner } from '../common/loading_spinner';
import { ItemTableView, type ItemTableColumn } from '../views/item_table_view';

const mostViewedTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
];

const statusTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
    { attributeKey: 'AP', kind: 'attribute', label: 'AP' },
    { attributeKey: 'Date', kind: 'attribute', label: 'Date' },
];

const overdueTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Status', kind: 'attribute', label: 'Status' },
    { attributeKey: 'Date', kind: 'attribute', label: 'Date' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
];

const backlogTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
    { attributeKey: 'AP', kind: 'attribute', label: 'AP' },
];

const recentTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Status', kind: 'attribute', label: 'Status' },
    { attributeKey: 'Date', kind: 'attribute', label: 'Date' },
];

const itemApi = new ItemAPI('/api');

const DAYS_STORAGE_KEY = 'zealot_analysis_days';
const DAYS_OPTIONS = [7, 30, 90, 365] as const;
type DaysOption = typeof DAYS_OPTIONS[number];

function loadStoredDays(): DaysOption {
    const stored = localStorage.getItem(DAYS_STORAGE_KEY);
    const parsed = stored ? parseInt(stored, 10) : NaN;
    return (DAYS_OPTIONS as readonly number[]).includes(parsed) ? parsed as DaysOption : 30;
}

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

    static groupByType(items: Item[]): Record<string, number> {
        const categories: Record<string, number> = {};
        items.forEach(item => {
            const name = item.Types[0]?.Name ?? 'Untyped';
            categories[name] = (categories[name] ?? 0) + 1;
        });
        return categories;
    }

    static computeStreakStats(items: Item[], days: number): { current: number; best: number; avgScore: number } {
        const scores = AnalysisUtils.computeScoreTimeSeries(items, days);
        const completedCounts = AnalysisUtils.computeCompletedTimeSeries(items, days);

        let current = 0;
        let best = 0;
        let run = 0;
        let scoreSum = 0;
        let scoreDays = 0;

        for (let i = 0; i < scores.y.length; i++) {
            const completed = completedCounts.y[i] ?? 0;
            const score = scores.y[i] ?? 0;
            if (completed > 0) {
                run++;
                if (run > best) best = run;
            } else {
                run = 0;
            }
            if (score > 0) {
                scoreSum += score;
                scoreDays++;
            }
        }

        // Walk back from today to find current streak
        current = 0;
        for (let i = scores.y.length - 1; i >= 0; i--) {
            if ((completedCounts.y[i] ?? 0) > 0) current++;
            else break;
        }

        const avgScore = scoreDays > 0 ? Math.round(scoreSum / scoreDays) : 0;
        return { current, best, avgScore };
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

function renderTabBar(active: 'analysis' | 'specify' | 'working' | 'recent' | 'backlog' | 'overdue' | 'most_viewed'): HTMLElement {
    const bar = document.createElement('div');
    bar.className = 'analysis-tabs';

    const tabs: { label: string; key: 'analysis' | 'specify' | 'working' | 'recent' | 'backlog' | 'overdue' | 'most_viewed' }[] = [
        { label: 'Analysis', key: 'analysis' },
        { label: 'Specify', key: 'specify' },
        { label: 'Working', key: 'working' },
        { label: 'Recent', key: 'recent' },
        { label: 'Backlog', key: 'backlog' },
        { label: 'Overdue', key: 'overdue' },
        { label: 'Most Viewed', key: 'most_viewed' },
    ];

    tabs.forEach(tab => {
        const btn = document.createElement('button');
        btn.textContent = tab.label;
        if (tab.key === active) btn.classList.add('is-active');
        btn.addEventListener('click', () => {
            const nav = getNavigator();
            if (tab.key === 'analysis') nav.openAnalysis();
            else if (tab.key === 'specify') nav.openAnalysisSpecify();
            else if (tab.key === 'working') nav.openAnalysisWorking();
            else if (tab.key === 'recent') nav.openAnalysisRecent();
            else if (tab.key === 'backlog') nav.openAnalysisBacklog();
            else if (tab.key === 'most_viewed') nav.openAnalysisMostViewed();
            else nav.openAnalysisOverdue();
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

    const typesChart = new graphs.PieChartView();
    typesChart.className = 'analysis-chart';
    typesChart.init({ caption: 'Types', items: AnalysisUtils.groupByType(items) });

    row.appendChild(statusChart);
    row.appendChild(priorityChart);
    row.appendChild(apChart);
    row.appendChild(typesChart);
    return row;
}

function renderCharts(items: Item[], days: number, container: HTMLElement): void {
    const count = document.createElement('div');
    count.textContent = `Items: ${items.length}`;
    container.appendChild(count);

    container.appendChild(renderPieRow(items));

    const parentRow = document.createElement('div');
    parentRow.className = 'analysis-charts-row';
    const parentChart = new graphs.PieChartView();
    parentChart.className = 'analysis-chart';
    parentChart.init({ caption: 'Parents', items: AnalysisUtils.groupBySum(items, 'Parent') });
    parentRow.appendChild(parentChart);
    container.appendChild(parentRow);

    const streakStats = AnalysisUtils.computeStreakStats(items, days);
    const statsRow = document.createElement('div');
    statsRow.className = 'analysis-stats-row';
    const streakChip = document.createElement('span');
    streakChip.textContent = `Streak: ${streakStats.current} days`;
    const bestChip = document.createElement('span');
    bestChip.textContent = `Best: ${streakStats.best} days`;
    const avgChip = document.createElement('span');
    avgChip.textContent = `Avg score: ${streakStats.avgScore}`;
    statsRow.appendChild(streakChip);
    statsRow.appendChild(bestChip);
    statsRow.appendChild(avgChip);
    container.appendChild(statsRow);

    const scoreData = AnalysisUtils.computeScoreTimeSeries(items, days);
    const scoreGraph = new graphs.LineGraphView();
    scoreGraph.init({ caption: 'Scorecard', x: scoreData.x, y: scoreData.y });
    container.appendChild(scoreGraph);

    const completedData = AnalysisUtils.computeCompletedTimeSeries(items, days);
    const completedGraph = new graphs.LineGraphView();
    completedGraph.init({ caption: 'Completed Goals', x: completedData.x, y: completedData.y });
    container.appendChild(completedGraph);
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

        let days = loadStoredDays();

        const subheading = document.createElement('h2');
        subheading.textContent = `Last ${days} Days`;
        wrapper.appendChild(subheading);

        const select = document.createElement('select');
        DAYS_OPTIONS.forEach(opt => {
            const option = document.createElement('option');
            option.value = String(opt);
            option.textContent = `${opt} days`;
            if (opt === days) option.selected = true;
            select.appendChild(option);
        });
        wrapper.appendChild(select);

        const chartsContainer = document.createElement('div');
        wrapper.appendChild(chartsContainer);
        this.appendChild(wrapper);

        const loadCharts = async (d: number) => {
            const spinner = new LoadingSpinner();
            chartsContainer.innerHTML = '';
            chartsContainer.appendChild(spinner);

            const items = await AnalysisUtils.getItemsByDays(d);
            chartsContainer.removeChild(spinner);
            renderCharts(items, d, chartsContainer);
        };

        select.addEventListener('change', () => {
            const val = parseInt(select.value, 10) as DaysOption;
            days = val;
            localStorage.setItem(DAYS_STORAGE_KEY, String(val));
            subheading.textContent = `Last ${val} Days`;
            loadCharts(val);
        });

        await loadCharts(days);
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

export class RecentScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('recent'));

        const heading = document.createElement('h1');
        heading.textContent = 'Recent';
        wrapper.appendChild(heading);

        const tableContainer = document.createElement('div');
        wrapper.appendChild(tableContainer);

        const loadMoreBtn = document.createElement('button');
        loadMoreBtn.textContent = 'Load More';
        wrapper.appendChild(loadMoreBtn);

        this.appendChild(wrapper);

        const PAGE_SIZE = 30;
        let offset = 0;
        let allItems: Item[] = [];

        const loadPage = async () => {
            const spinner = new LoadingSpinner();
            wrapper.insertBefore(spinner, loadMoreBtn);
            loadMoreBtn.disabled = true;

            const page = await itemApi.GetRecent(PAGE_SIZE, offset);
            wrapper.removeChild(spinner);
            loadMoreBtn.disabled = false;

            allItems = [...allItems, ...page];
            offset += page.length;

            tableContainer.innerHTML = '';
            const table = new ItemTableView();
            table.init({ columns: recentTableColumns, items: allItems, emptyMessage: 'No recent items.' });
            tableContainer.appendChild(table);

            if (page.length < PAGE_SIZE) {
                loadMoreBtn.remove();
            }
        };

        loadMoreBtn.addEventListener('click', loadPage);

        await loadPage();
    }
}

customElements.define('recent-screen', RecentScreen);

export class BacklogScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('backlog'));

        const heading = document.createElement('h1');
        heading.textContent = 'Backlog';
        wrapper.appendChild(heading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const allToDo = await AnalysisUtils.getItemsByStatus('To Do');
        const items = allToDo.filter(
            i => !i.Attributes?.['Date'] && !i.Attributes?.['Week']
        );
        wrapper.removeChild(spinner);

        heading.textContent = `Backlog (${items.length})`;

        const table = new ItemTableView();
        table.init({ columns: backlogTableColumns, items, emptyMessage: 'No items in Backlog.' });
        wrapper.appendChild(table);
    }
}

customElements.define('backlog-screen', BacklogScreen);

export class OverdueScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('overdue'));

        const heading = document.createElement('h1');
        heading.textContent = 'Overdue';
        wrapper.appendChild(heading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const today = DateTime.now().toISODate()!;
        const all = await itemApi.Filter([{ key: 'Date', op: 'lt', value: today, list_mode: 'any' }]);
        const items = all.filter(i => {
            const status = i.Attributes?.['Status'];
            return status === 'To Do' || status === 'Working';
        });
        wrapper.removeChild(spinner);

        heading.textContent = `Overdue (${items.length})`;

        const table = new ItemTableView();
        table.init({ columns: overdueTableColumns, items, emptyMessage: 'No overdue items.' });
        wrapper.appendChild(table);
    }
}

customElements.define('overdue-screen', OverdueScreen);

export class MostViewedScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '';
        const wrapper = document.createElement('div');
        wrapper.className = 'analysis-screen';

        wrapper.appendChild(renderTabBar('most_viewed'));

        const heading = document.createElement('h1');
        heading.textContent = 'Most Viewed';
        wrapper.appendChild(heading);

        const spinner = new LoadingSpinner();
        wrapper.appendChild(spinner);
        this.appendChild(wrapper);

        const entries = await itemApi.GetMostViewed(50);
        wrapper.removeChild(spinner);

        heading.textContent = `Most Viewed (${entries.length})`;

        if (entries.length === 0) {
            const empty = document.createElement('p');
            empty.textContent = 'No views recorded yet. Open some items to start tracking.';
            wrapper.appendChild(empty);
            return;
        }

        const listEl = document.createElement('table');
        listEl.className = 'item-table';

        const thead = document.createElement('thead');
        const headerRow = document.createElement('tr');
        ['Title', 'Type', 'Views'].forEach(label => {
            const th = document.createElement('th');
            th.textContent = label;
            headerRow.appendChild(th);
        });
        thead.appendChild(headerRow);
        listEl.appendChild(thead);

        const tbody = document.createElement('tbody');
        entries.forEach(({ item, viewCount }) => {
            const row = document.createElement('tr');
            row.style.cursor = 'pointer';
            row.addEventListener('click', () => getNavigator().openItemById(item.ItemID));

            const titleCell = document.createElement('td');
            titleCell.textContent = item.DisplayTitle;
            row.appendChild(titleCell);

            const typeCell = document.createElement('td');
            typeCell.textContent = item.Types[0]?.Name ?? '';
            row.appendChild(typeCell);

            const viewCell = document.createElement('td');
            viewCell.textContent = String(viewCount);
            row.appendChild(viewCell);

            tbody.appendChild(row);
        });
        listEl.appendChild(tbody);
        wrapper.appendChild(listEl);
    }
}

customElements.define('most-viewed-screen', MostViewedScreen);

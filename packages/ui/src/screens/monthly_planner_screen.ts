import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { PlannerAPI } from '@zealot/api/src/planner';
import { icons } from '@zealot/content';
import { LoadingSpinner } from '../common/loading_spinner';
import {
    createPlannerErrorCard,
    createPlannerHeader,
    createPlannerSection,
    currentMonth,
    formatMonthCode,
    formatMonthTitle,
    mountPlannerTable,
    parseMonthCode,
    renderPlannerMessage,
} from './planner_shared';

const plannerApi = new PlannerAPI('/api');

export class MonthlyPlannerScreen extends BaseElementEmpty {
    private date: string | null = null;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        const date = this.date ? parseMonthCode(this.date) : currentMonth();

        this.className = 'planner-screen';
        this.innerHTML = '';

        if (!date) {
            this.appendChild(createPlannerErrorCard(
                'Invalid month. Use YYYY-MM.',
                'Go to this month',
                () => getNavigator().openPlanner('monthly'),
            ));
            return;
        }

        const header = createPlannerHeader(formatMonthTitle(date), [
            {
                iconURL: icons.back,
                label: 'Previous Month',
                onClick: () => getNavigator().openPlanner('monthly', formatMonthCode(date.minus({ months: 1 }))),
            },
            {
                iconURL: icons.month,
                label: 'This Month',
                onClick: () => getNavigator().openPlanner('monthly'),
            },
            {
                iconURL: icons.forward,
                label: 'Next Month',
                onClick: () => getNavigator().openPlanner('monthly', formatMonthCode(date.plus({ months: 1 }))),
            },
        ]);

        const items = createPlannerSection('Month Items');
        items.body.appendChild(new LoadingSpinner());
        this.append(header, items.section);

        try {
            const data = await plannerApi.GetForMonth(date);
            if (renderId !== this.renderId) {
                return;
            }

            mountPlannerTable(items.body, {
                createRow: {
                    defaultAttributes: {
                        Month: date.month,
                        Priority: 3,
                        Status: 'To Do',
                        Year: date.year,
                    },
                    enabled: true,
                    submitLabel: 'Add Item',
                },
                emptyMessage: 'No items scheduled for this month.',
                items: data,
            });
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            renderPlannerMessage(items.body, this._messageForError(error, 'Failed to load items.'), 'error');
        }
    }

    init(date: string): this {
        this.date = date;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    private _messageForError(error: unknown, fallback: string): string {
        return error instanceof Error && error.message ? error.message : fallback;
    }
}

if (!customElements.get('monthly-planner-screen')) {
    customElements.define('monthly-planner-screen', MonthlyPlannerScreen);
}

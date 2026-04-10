import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { PlannerAPI } from '@zealot/api/src/planner';
import { icons } from '@zealot/content';
import { LoadingSpinner } from '../common/loading_spinner';
import {
    createPlannerErrorCard,
    createPlannerHeader,
    createPlannerSection,
    currentYear,
    formatYearCode,
    formatYearTitle,
    mountPlannerTable,
    parseYearCode,
    renderPlannerMessage,
} from './planner_shared';

const plannerApi = new PlannerAPI('/api');

export class AnnualPlannerScreen extends BaseElementEmpty {
    private year: string | null = null;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        const date = this.year ? parseYearCode(this.year) : currentYear();

        this.className = 'planner-screen';
        this.innerHTML = '';

        if (!date) {
            this.appendChild(createPlannerErrorCard(
                'Invalid year. Use YYYY.',
                'Go to this year',
                () => getNavigator().openPlanner('annual'),
            ));
            return;
        }

        const header = createPlannerHeader(formatYearTitle(date), [
            {
                iconURL: icons.back,
                label: 'Previous Year',
                onClick: () => getNavigator().openPlanner('annual', formatYearCode(date.minus({ years: 1 }))),
            },
            {
                iconURL: icons.sun,
                label: 'This Year',
                onClick: () => getNavigator().openPlanner('annual'),
            },
            {
                iconURL: icons.forward,
                label: 'Next Year',
                onClick: () => getNavigator().openPlanner('annual', formatYearCode(date.plus({ years: 1 }))),
            },
        ]);

        const items = createPlannerSection('Year Items');
        items.body.appendChild(new LoadingSpinner());
        this.append(header, items.section);

        try {
            const data = await plannerApi.GetForYear(date);
            if (renderId !== this.renderId) {
                return;
            }

            mountPlannerTable(items.body, {
                createRow: {
                    defaultAttributes: {
                        Priority: 3,
                        Status: 'To Do',
                        Year: date.year,
                    },
                    enabled: true,
                    submitLabel: 'Add Item',
                },
                emptyMessage: 'No items scheduled for this year.',
                items: data,
            });
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            renderPlannerMessage(items.body, this._messageForError(error, 'Failed to load items.'), 'error');
        }
    }

    init(year: string): this {
        this.year = year;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    private _messageForError(error: unknown, fallback: string): string {
        return error instanceof Error && error.message ? error.message : fallback;
    }
}

if (!customElements.get('annual-planner-screen')) {
    customElements.define('annual-planner-screen', AnnualPlannerScreen);
}

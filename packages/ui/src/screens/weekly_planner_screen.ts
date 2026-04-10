import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { PlannerAPI } from '@zealot/api/src/planner';
import { icons } from '@zealot/content';
import { LoadingSpinner } from '../common/loading_spinner';
import {
    createPlannerErrorCard,
    createPlannerHeader,
    createPlannerSection,
    currentWeek,
    formatIsoWeek,
    formatWeekTitle,
    mountPlannerTable,
    parseIsoWeek,
    renderPlannerMessage,
} from './planner_shared';

const plannerApi = new PlannerAPI('/api');

export class WeeklyPlannerScreen extends BaseElementEmpty {
    private date: string | null = null;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        const date = this.date ? parseIsoWeek(this.date) : currentWeek();

        this.className = 'planner-screen';
        this.innerHTML = '';

        if (!date) {
            this.appendChild(createPlannerErrorCard(
                'Invalid week. Use YYYY-Www.',
                'Go to this week',
                () => getNavigator().openPlanner('weekly'),
            ));
            return;
        }

        const header = createPlannerHeader(formatWeekTitle(date), [
            {
                iconURL: icons.back,
                label: 'Previous Week',
                onClick: () => getNavigator().openPlanner('weekly', formatIsoWeek(date.minus({ weeks: 1 }))),
            },
            {
                iconURL: icons.week,
                label: 'This Week',
                onClick: () => getNavigator().openPlanner('weekly'),
            },
            {
                iconURL: icons.forward,
                label: 'Next Week',
                onClick: () => getNavigator().openPlanner('weekly', formatIsoWeek(date.plus({ weeks: 1 }))),
            },
        ]);

        const items = createPlannerSection('Week Items');
        items.body.appendChild(new LoadingSpinner());
        this.append(header, items.section);

        try {
            const data = await plannerApi.GetForWeek(date);
            if (renderId !== this.renderId) {
                return;
            }

            mountPlannerTable(items.body, {
                createRow: {
                    defaultAttributes: {
                        Priority: 3,
                        Status: 'To Do',
                        Week: formatIsoWeek(date),
                    },
                    enabled: true,
                    submitLabel: 'Add Item',
                },
                emptyMessage: 'No items scheduled for this week.',
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

if (!customElements.get('weekly-planner-screen')) {
    customElements.define('weekly-planner-screen', WeeklyPlannerScreen);
}

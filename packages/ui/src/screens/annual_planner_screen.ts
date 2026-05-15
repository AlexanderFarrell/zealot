import { BaseElementEmpty, getNavigator, unregisterDropZonesIn } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
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
    mountPlannerCardList,
    parseYearCode,
    renderPlannerMessage,
} from './planner_shared';

const attrApi = new AttributeAPI('/api');

const plannerApi = new PlannerAPI('/api');

export class AnnualPlannerScreen extends BaseElementEmpty {
    private year: string | null = null;
    private renderId = 0;
    private headerEl: HTMLElement | null = null;

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

        if (this.headerEl) {
            unregisterDropZonesIn(this.headerEl);
            this.headerEl = null;
        }

        const header = createPlannerHeader(
            formatYearTitle(date),
            [
                {
                    iconURL: icons.back,
                    label: 'Previous Year',
                    onClick: () => getNavigator().openPlanner('annual', formatYearCode(date.minus({ years: 1 }))),
                    onDrop: (id) => { void attrApi.set_value(id, 'Year', date.minus({ years: 1 }).year); },
                },
                {
                    iconURL: icons.sun,
                    label: 'This Year',
                    onClick: () => getNavigator().openPlanner('annual'),
                    onDrop: (id) => { void attrApi.set_value(id, 'Year', currentYear().year); },
                },
                {
                    iconURL: icons.forward,
                    label: 'Next Year',
                    onClick: () => getNavigator().openPlanner('annual', formatYearCode(date.plus({ years: 1 }))),
                    onDrop: (id) => { void attrApi.set_value(id, 'Year', date.plus({ years: 1 }).year); },
                },
            ],
            [
                {
                    iconURL: icons.addNote,
                    label: 'Journal',
                    onClick: () => { getNavigator().openItem(formatYearCode(date)); },
                },
            ],
        );

        const items = createPlannerSection('Year Items');
        this.headerEl = header;
        items.body.appendChild(new LoadingSpinner());
        this.append(header, items.section);

        try {
            const data = await plannerApi.GetForYear(date);
            if (renderId !== this.renderId) {
                return;
            }

            mountPlannerCardList(items.body, {
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

    disconnectedCallback(): void {
        if (this.headerEl) {
            unregisterDropZonesIn(this.headerEl);
            this.headerEl = null;
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

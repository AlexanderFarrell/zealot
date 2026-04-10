import { BaseElementEmpty, Popups, getNavigator } from '@websoil/engine';
import { PlannerAPI } from '@zealot/api/src/planner';
import { RepeatAPI } from '@zealot/api/src/repeat';
import { icons } from '@zealot/content';
import { RepeatStatus, type RepeatEntry, type RepeatStatusType } from '@zealot/domain/src/repeat';
import { LoadingSpinner } from '../common/loading_spinner';
import { CommentsView } from '../views/comments_view';
import {
    createPlannerErrorCard,
    createPlannerHeader,
    createPlannerSection,
    currentDay,
    formatDayTitle,
    formatIsoDate,
    mountPlannerTable,
    parseIsoDate,
    renderPlannerMessage,
} from './planner_shared';

const plannerApi = new PlannerAPI('/api');
const repeatApi = new RepeatAPI('/api');
const repeatGroups = ['Morning', 'Afternoon', 'Evening', 'Anytime'] as const;
const repeatLabels: Record<RepeatStatusType, string> = {
    Alternate: 'Alternate',
    Complete: 'Done',
    'Not Complete': 'None',
    Skip: 'Skipped',
};

export class DailyPlannerScreen extends BaseElementEmpty {
    private date: string | null = null;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        const date = this.date ? parseIsoDate(this.date) : currentDay();

        this.className = 'planner-screen';
        this.innerHTML = '';

        if (!date) {
            this.appendChild(createPlannerErrorCard(
                'Invalid day. Use YYYY-MM-DD.',
                'Go to today',
                () => getNavigator().openPlanner('daily'),
            ));
            return;
        }

        const header = createPlannerHeader(formatDayTitle(date), [
            {
                iconURL: icons.back,
                label: 'Previous Day',
                onClick: () => getNavigator().openPlanner('daily', formatIsoDate(date.minus({ days: 1 }))),
            },
            {
                iconURL: icons.today,
                label: 'Today',
                onClick: () => getNavigator().openPlanner('daily'),
            },
            {
                iconURL: icons.forward,
                label: 'Next Day',
                onClick: () => getNavigator().openPlanner('daily', formatIsoDate(date.plus({ days: 1 }))),
            },
        ]);

        const items = createPlannerSection('Items');
        const repeats = createPlannerSection('Repeats');
        const comments = createPlannerSection('Comments');
        items.body.appendChild(new LoadingSpinner());
        repeats.body.appendChild(new LoadingSpinner());
        comments.body.appendChild(new CommentsView().init({
            scope: { kind: 'day', date },
        }));

        this.append(header, items.section, repeats.section, comments.section);

        const [itemsResult, repeatsResult] = await Promise.allSettled([
            plannerApi.GetForDay(date),
            repeatApi.GetForDay(date),
        ]);

        if (renderId !== this.renderId) {
            return;
        }

        if (itemsResult.status === 'fulfilled') {
            mountPlannerTable(items.body, {
                createRow: {
                    defaultAttributes: {
                        Date: formatIsoDate(date),
                        Priority: 3,
                        Status: 'To Do',
                    },
                    enabled: true,
                    submitLabel: 'Add Item',
                },
                emptyMessage: 'No items scheduled for this day.',
                items: itemsResult.value,
            });
        } else {
            renderPlannerMessage(items.body, this._messageForError(itemsResult.reason, 'Failed to load items.'), 'error');
        }

        if (repeatsResult.status === 'fulfilled') {
            this._renderRepeats(repeats.body, repeatsResult.value, date);
        } else {
            renderPlannerMessage(repeats.body, this._messageForError(repeatsResult.reason, 'Failed to load repeats.'), 'error');
        }
    }

    init(date: string): this {
        this.date = date;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    private _renderRepeats(container: HTMLElement, entries: RepeatEntry[], date: import('luxon').DateTime): void {
        container.innerHTML = '';

        if (entries.length === 0) {
            renderPlannerMessage(container, 'No repeats scheduled.');
            return;
        }

        const buckets = new Map<string, RepeatEntry[]>();
        repeatGroups.forEach((label) => buckets.set(label, []));

        entries.forEach((entry) => {
            const raw = entry.Item.Attributes['Time of Day'];
            const bucket = typeof raw === 'string' && buckets.has(raw) ? raw : 'Anytime';
            buckets.get(bucket)!.push(entry);
        });

        const groups = document.createElement('div');
        groups.className = 'planner-repeat-groups';

        repeatGroups.forEach((label) => {
            const items = buckets.get(label) ?? [];
            if (items.length === 0) {
                return;
            }

            const section = document.createElement('section');
            section.className = 'planner-repeat-group';

            const heading = document.createElement('h3');
            heading.textContent = label;
            section.appendChild(heading);

            items.forEach((entry) => {
                section.appendChild(this._buildRepeatRow(entry, date));
            });

            groups.appendChild(section);
        });

        container.appendChild(groups);
    }

    private _buildRepeatRow(entry: RepeatEntry, date: import('luxon').DateTime): HTMLElement {
        const row = document.createElement('div');
        row.className = 'planner-repeat-row';

        const title = document.createElement('button');
        title.type = 'button';
        title.className = 'planner-repeat-title';
        title.textContent = entry.Item.DisplayTitle;
        title.addEventListener('click', () => {
            getNavigator().openItemById(entry.Item.ItemID);
        });

        const controls = document.createElement('div');
        controls.className = 'planner-repeat-controls';

        const status = document.createElement('select');
        RepeatStatus.forEach((value) => {
            const option = document.createElement('option');
            option.value = value;
            option.textContent = repeatLabels[value] ?? value;
            status.appendChild(option);
        });
        status.value = entry.Status;

        const comment = document.createElement('input');
        comment.type = 'text';
        comment.placeholder = 'Comment';
        comment.value = entry.Comment;

        let saving = false;
        const save = async (): Promise<void> => {
            if (saving) {
                return;
            }

            const nextStatus = status.value as RepeatStatusType;
            const nextComment = comment.value;

            if (nextStatus === entry.Status && nextComment === entry.Comment) {
                return;
            }

            const previousStatus = entry.Status;
            const previousComment = entry.Comment;

            saving = true;
            status.disabled = true;
            comment.disabled = true;

            try {
                await repeatApi.SetStatus({
                    comment: nextComment,
                    date,
                    item_id: entry.Item.ItemID,
                    status: nextStatus,
                });
                entry.Status = nextStatus;
                entry.Comment = nextComment;
            } catch (error) {
                status.value = previousStatus;
                comment.value = previousComment;
                Popups.add_error(this._messageForError(error, 'Failed to save repeat status.'));
            } finally {
                saving = false;
                status.disabled = false;
                comment.disabled = false;
            }
        };

        status.addEventListener('change', () => {
            void save();
        });
        comment.addEventListener('blur', () => {
            void save();
        });

        controls.append(status, comment);
        row.append(title, controls);
        return row;
    }

    private _messageForError(error: unknown, fallback: string): string {
        return error instanceof Error && error.message ? error.message : fallback;
    }
}

if (!customElements.get('daily-planner-screen')) {
    customElements.define('daily-planner-screen', DailyPlannerScreen);
}

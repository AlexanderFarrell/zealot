import { Popups, getNavigator } from '@websoil/engine';
import { RepeatAPI } from '@zealot/api/src/repeat';
import { RepeatStatus, type RepeatEntry, type RepeatStatusType } from '@zealot/domain/src/repeat';

const repeatApi = new RepeatAPI('/api');
const repeatGroups = ['Morning', 'Afternoon', 'Evening', 'Anytime'] as const;
const repeatLabels: Record<RepeatStatusType, string> = {
    Alternate: 'Alternate',
    Complete: 'Done',
    'Not Complete': 'None',
    Skip: 'Skipped',
};

export class RepeatsSidebarView extends HTMLElement {
    init(entries: RepeatEntry[], date: import('luxon').DateTime): this {
        this.innerHTML = '';

        const container = document.createElement('div');
        container.className = 'sidebar-repeats';

        const title = document.createElement('p');
        title.className = 'sidebar-repeats-title';
        title.textContent = 'Repeats';
        container.appendChild(title);

        if (entries.length === 0) {
            const msg = document.createElement('p');
            msg.className = 'tool-muted';
            msg.textContent = 'No repeats scheduled.';
            container.appendChild(msg);
            this.appendChild(container);
            return this;
        }

        const buckets = new Map<string, RepeatEntry[]>();
        repeatGroups.forEach((label) => buckets.set(label, []));

        entries.forEach((entry) => {
            const raw = entry.Item.Attributes['Time of Day'];
            const bucket = typeof raw === 'string' && buckets.has(raw) ? raw : 'Anytime';
            buckets.get(bucket)!.push(entry);
        });

        repeatGroups.forEach((label) => {
            const items = buckets.get(label) ?? [];
            if (items.length === 0) {
                return;
            }

            const section = document.createElement('section');
            section.className = 'sidebar-repeat-group';

            const header = document.createElement('div');
            header.className = 'sidebar-repeat-group-header';

            const headerLabel = document.createElement('span');
            headerLabel.textContent = label;

            const countEl = document.createElement('span');
            countEl.className = 'sidebar-repeat-count';
            const updateCount = () => {
                const done = items.filter((e) => e.Status === 'Complete').length;
                countEl.textContent = `${done} / ${items.length}`;
            };
            updateCount();

            header.append(headerLabel, countEl);
            section.appendChild(header);

            items.forEach((entry) => {
                section.appendChild(this._buildRow(entry, date, updateCount));
            });

            container.appendChild(section);
        });

        this.appendChild(container);
        return this;
    }

    private _buildRow(
        entry: RepeatEntry,
        date: import('luxon').DateTime,
        onStatusChange: () => void,
    ): HTMLElement {
        const row = document.createElement('div');
        row.className = 'sidebar-repeat-row' + (entry.Status === 'Complete' ? ' is-complete' : '');

        const title = document.createElement('button');
        title.type = 'button';
        title.className = 'sidebar-repeat-title';
        title.textContent = entry.Item.DisplayTitle;
        title.addEventListener('click', () => {
            getNavigator().openItemById(entry.Item.ItemID);
        });

        const statusEl = document.createElement('select');
        statusEl.className = 'sidebar-repeat-status';
        RepeatStatus.forEach((value) => {
            const option = document.createElement('option');
            option.value = value;
            option.textContent = repeatLabels[value] ?? value;
            statusEl.appendChild(option);
        });
        statusEl.value = entry.Status;

        let saving = false;
        statusEl.addEventListener('change', () => {
            if (saving) {
                return;
            }

            const nextStatus = statusEl.value as RepeatStatusType;
            if (nextStatus === entry.Status) {
                return;
            }

            const previousStatus = entry.Status;
            saving = true;
            statusEl.disabled = true;

            void repeatApi
                .SetStatus({
                    comment: entry.Comment,
                    date,
                    item_id: entry.Item.ItemID,
                    status: nextStatus,
                })
                .then(() => {
                    entry.Status = nextStatus;
                    row.classList.toggle('is-complete', nextStatus === 'Complete');
                    onStatusChange();
                })
                .catch((error: unknown) => {
                    statusEl.value = previousStatus;
                    Popups.add_error(
                        error instanceof Error && error.message
                            ? error.message
                            : 'Failed to save repeat status.',
                    );
                })
                .finally(() => {
                    saving = false;
                    statusEl.disabled = false;
                });
        });

        row.append(title, statusEl);
        return row;
    }
}

if (!customElements.get('repeats-sidebar-view')) {
    customElements.define('repeats-sidebar-view', RepeatsSidebarView);
}

import { Popups, getNavigator } from '@websoil/engine';
import { RepeatAPI } from '@zealot/api/src/repeat';
import { RepeatStatus, type RepeatEntry, type RepeatStatusType } from '@zealot/domain/src/repeat';
import { renderItemTitle } from './item_title';

const repeatApi = new RepeatAPI('/api');
const repeatGroups = ['Morning', 'Afternoon', 'Evening', 'Anytime'] as const;
const repeatIcons: Record<RepeatStatusType, string> = {
    Alternate: '⇄',
    Complete: '✓',
    'Not Complete': '○',
    Skip: '–',
};
const repeatMenuStatuses: RepeatStatusType[] = ['Skip', 'Alternate', 'Not Complete'];
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
        renderItemTitle(title, entry.Item, { className: 'sidebar-repeat-title-label' });
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

        // Row controls (Z238, Z253): one tap or click completes, the ⋯ menu holds the rest.
        const check = document.createElement('button');
        check.type = 'button';
        check.className = 'sidebar-repeat-check';
        const more = document.createElement('button');
        more.type = 'button';
        more.className = 'sidebar-repeat-more';
        more.textContent = '⋯';
        more.setAttribute('aria-label', 'More statuses');
        more.title = 'Skip, Alternate, Not done';
        more.setAttribute('aria-expanded', 'false');
        const menu = document.createElement('div');
        menu.className = 'sidebar-repeat-menu';
        menu.hidden = true;

        const paint = (): void => {
            const status = entry.Status as RepeatStatusType;
            check.textContent = repeatIcons[status] ?? '○';
            check.dataset.status = status;
            check.setAttribute('aria-label', status === 'Complete' ? 'Mark not done' : 'Mark done');
            check.title = status === 'Complete' ? 'Mark not done' : 'Mark done';
            check.setAttribute('aria-pressed', String(status === 'Complete'));
            row.dataset.status = status;
            row.classList.toggle('is-complete', status === 'Complete');
            statusEl.value = status;
        };

        let saving = false;
        const save = (nextStatus: RepeatStatusType): void => {
            if (saving || nextStatus === entry.Status) {
                return;
            }

            saving = true;
            statusEl.disabled = true;
            check.disabled = true;
            more.disabled = true;

            void repeatApi
                .SetStatus({
                    comment: entry.Comment,
                    date,
                    item_id: entry.Item.ItemID,
                    status: nextStatus,
                })
                .then(() => {
                    entry.Status = nextStatus;
                    paint();
                    onStatusChange();
                    row.dispatchEvent(new CustomEvent('repeat-status-change', { bubbles: true }));
                })
                .catch((error: unknown) => {
                    paint();
                    Popups.add_error(
                        error instanceof Error && error.message
                            ? error.message
                            : 'Failed to save repeat status.',
                    );
                })
                .finally(() => {
                    saving = false;
                    statusEl.disabled = false;
                    check.disabled = false;
                    more.disabled = false;
                });
        };

        statusEl.addEventListener('change', () => {
            save(statusEl.value as RepeatStatusType);
        });
        check.addEventListener('click', () => {
            save(entry.Status === 'Complete' ? 'Not Complete' : 'Complete');
        });

        repeatMenuStatuses.forEach((value) => {
            const option = document.createElement('button');
            option.type = 'button';
            option.className = 'sidebar-repeat-menu-option';
            option.textContent = value === 'Not Complete' ? 'Not done' : (repeatLabels[value] ?? value);
            option.addEventListener('click', () => {
                menu.hidden = true;
                more.setAttribute('aria-expanded', 'false');
                save(value);
            });
            menu.appendChild(option);
        });
        more.addEventListener('click', () => {
            menu.hidden = !menu.hidden;
            more.setAttribute('aria-expanded', String(!menu.hidden));
        });

        paint();
        row.append(check, title, more, statusEl, menu);
        return row;
    }
}

if (!customElements.get('repeats-sidebar-view')) {
    customElements.define('repeats-sidebar-view', RepeatsSidebarView);
}

import { Popups } from '@websoil/engine';
import { StatisticAPI } from '@zealot/api/src/statistic';
import type { Item } from '@zealot/domain/src/item';
import type {
    CreateStatisticEntryDto,
    StatisticEntry,
    UpdateStatisticEntryDto,
} from '@zealot/domain/src/statistic';
import { DateTime } from 'luxon';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';
import { ItemPickerInput } from './item_picker_input';

const statisticApi = new StatisticAPI('/api');

function localInputValue(value = DateTime.local()): string {
    return value.toLocal().toFormat("yyyy-MM-dd'T'HH:mm");
}

function toUtc(value: string): string | undefined {
    if (!value) return undefined;
    const parsed = DateTime.fromISO(value);
    return parsed.isValid ? parsed.toUTC().toISO() ?? undefined : undefined;
}

export class StatisticsView extends HTMLElement {
    private item: Item | null = null;
    private entries: StatisticEntry[] = [];
    private nextOffset: number | null = null;
    private loading = false;
    private editingId: number | null = null;

    init(item: Item): this {
        this.item = item;
        if (this.isConnected) void this.load(false);
        return this;
    }

    connectedCallback(): void {
        if (this.item) void this.load(false);
    }

    private get unit(): string {
        return String(this.item?.Attributes['Unit'] ?? '').trim();
    }

    private get valueKind(): string {
        return String(this.item?.Attributes['Value Kind'] ?? 'Number');
    }

    private formatValue(value: number): string {
        return String(value) + (this.unit ? ' ' + this.unit : '');
    }

    private async load(append: boolean): Promise<void> {
        if (!this.item || this.loading) return;
        this.loading = true;
        if (!append) {
            this.entries = [];
            this.nextOffset = null;
            this.render();
        }
        try {
            const page = await statisticApi.GetEntries(this.item.ItemID, {
                limit: 20,
                offset: append ? this.nextOffset ?? 0 : 0,
            });
            this.entries = append ? [...this.entries, ...page.entries] : page.entries;
            this.nextOffset = page.nextOffset;
        } catch (error) {
            Popups.add_error(error instanceof Error ? error.message : 'Failed to load Statistics.');
        } finally {
            this.loading = false;
            this.render();
        }
    }

    private render(): void {
        this.innerHTML = '';
        if (!this.item) return;
        this.className = 'statistics-view';
        this.appendChild(this.buildComposer());

        if (this.loading && this.entries.length === 0) {
            this.appendChild(new LoadingSpinner());
            return;
        }

        const delta = document.createElement('p');
        delta.className = 'statistics-view-delta';
        if (this.entries.length >= 2) {
            const value = this.entries[0]!.Value - this.entries[1]!.Value;
            delta.textContent = 'Latest delta: ' + (value >= 0 ? '+' : '') + this.formatValue(value);
        } else {
            delta.textContent = 'Latest delta: —';
        }
        this.appendChild(delta);

        const list = document.createElement('div');
        list.className = 'statistics-view-list';
        if (this.entries.length === 0) {
            const empty = document.createElement('p');
            empty.textContent = 'No Statistic Entries yet.';
            list.appendChild(empty);
        } else {
            for (const entry of this.entries) {
                list.appendChild(this.editingId === entry.StatisticEntryID ? this.buildEditor(entry) : this.buildEntry(entry));
            }
        }
        this.appendChild(list);

        if (this.nextOffset != null) {
            const more = document.createElement('button');
            more.type = 'button';
            more.textContent = this.loading ? 'Loading…' : 'Load more';
            more.disabled = this.loading;
            more.addEventListener('click', () => void this.load(true));
            this.appendChild(more);
        }
    }

    private buildComposer(): HTMLElement {
        const form = document.createElement('form');
        form.className = 'statistics-view-composer';
        const value = document.createElement('input');
        value.type = 'number';
        value.step = this.valueKind === 'Ordinal' ? '1' : 'any';
        value.required = true;
        value.placeholder = this.valueKind + (this.unit ? ' (' + this.unit + ')' : '');
        value.setAttribute('aria-label', this.valueKind + ' value');
        const occurred = document.createElement('input');
        occurred.type = 'datetime-local';
        occurred.value = localInputValue();
        occurred.setAttribute('aria-label', 'Occurred at');
        const related = new ItemPickerInput();
        related.setAttribute('aria-label', 'Related item');
        const comment = document.createElement('input');
        comment.type = 'text';
        comment.placeholder = 'Optional comment';
        const submit = document.createElement('button');
        submit.type = 'submit';
        submit.textContent = 'Record';
        form.append(value, occurred, related, comment, submit);
        form.addEventListener('submit', event => {
            event.preventDefault();
            const numeric = Number(value.value);
            if (!Number.isFinite(numeric)) {
                Popups.add_error('Enter a finite numeric value.');
                return;
            }
            submit.disabled = true;
            const dto: CreateStatisticEntryDto = { value: numeric };
            const occurredAt = toUtc(occurred.value);
            if (occurredAt) dto.occurred_at = occurredAt;
            if (related.value != null) dto.related_item_id = related.value;
            if (comment.value.trim()) dto.comment = comment.value.trim();
            void statisticApi.Create(this.item!.ItemID, dto).then(entry => {
                this.entries = [entry, ...this.entries];
                if (this.nextOffset != null) this.nextOffset += 1;
                value.value = '';
                comment.value = '';
                related.value = null;
                occurred.value = localInputValue();
                this.render();
            }).catch(error => {
                Popups.add_error(error instanceof Error ? error.message : 'Failed to record Statistic Entry.');
                submit.disabled = false;
            });
        });
        return form;
    }

    private buildEntry(entry: StatisticEntry): HTMLElement {
        const card = document.createElement('article');
        card.className = 'statistics-view-entry';
        const main = document.createElement('div');
        const value = document.createElement('strong');
        value.textContent = this.formatValue(entry.Value);
        const time = document.createElement('time');
        time.dateTime = entry.OccurredAt.toISO() ?? '';
        time.textContent = entry.OccurredAt.toLocal().toLocaleString(DateTime.DATETIME_MED);
        main.append(value, time);
        if (entry.Comment) {
            const comment = document.createElement('p');
            comment.textContent = entry.Comment;
            main.appendChild(comment);
        }
        if (entry.RelatedItemID != null) {
            const related = document.createElement('small');
            related.textContent = 'Related item #' + entry.RelatedItemID;
            main.appendChild(related);
        }
        const actions = document.createElement('div');
        const edit = document.createElement('button');
        edit.type = 'button';
        edit.textContent = 'Edit';
        edit.addEventListener('click', () => {
            this.editingId = entry.StatisticEntryID;
            this.render();
        });
        const remove = document.createElement('button');
        remove.type = 'button';
        remove.textContent = 'Delete';
        remove.addEventListener('click', () => void this.removeEntry(entry));
        actions.append(edit, remove);
        card.append(main, actions);
        return card;
    }

    private buildEditor(entry: StatisticEntry): HTMLElement {
        const form = document.createElement('form');
        form.className = 'statistics-view-entry statistics-view-editor';
        const value = document.createElement('input');
        value.type = 'number';
        value.step = this.valueKind === 'Ordinal' ? '1' : 'any';
        value.value = String(entry.Value);
        const occurred = document.createElement('input');
        occurred.type = 'datetime-local';
        occurred.value = localInputValue(entry.OccurredAt);
        const related = new ItemPickerInput();
        related.value = entry.RelatedItemID;
        const comment = document.createElement('input');
        comment.value = entry.Comment ?? '';
        comment.placeholder = 'Optional comment';
        const save = document.createElement('button');
        save.type = 'submit';
        save.textContent = 'Save';
        const cancel = document.createElement('button');
        cancel.type = 'button';
        cancel.textContent = 'Cancel';
        cancel.addEventListener('click', () => {
            this.editingId = null;
            this.render();
        });
        form.append(value, occurred, related, comment, save, cancel);
        form.addEventListener('submit', event => {
            event.preventDefault();
            const numeric = Number(value.value);
            if (!Number.isFinite(numeric)) {
                Popups.add_error('Enter a finite numeric value.');
                return;
            }
            save.disabled = true;
            const dto: UpdateStatisticEntryDto = {
                value: numeric,
                related_item_id: related.value,
                comment: comment.value.trim() || null,
            };
            const occurredAt = toUtc(occurred.value);
            if (occurredAt) dto.occurred_at = occurredAt;
            void statisticApi.Update(entry.StatisticEntryID, dto).then(updated => {
                this.entries = this.entries
                    .map(current => current.StatisticEntryID === updated.StatisticEntryID ? updated : current)
                    .sort((a, b) => b.OccurredAt.toMillis() - a.OccurredAt.toMillis() || b.StatisticEntryID - a.StatisticEntryID);
                this.editingId = null;
                this.render();
            }).catch(error => {
                Popups.add_error(error instanceof Error ? error.message : 'Failed to edit Statistic Entry.');
                save.disabled = false;
            });
        });
        return form;
    }

    private async removeEntry(entry: StatisticEntry): Promise<void> {
        if (!await ConfirmDialog.show('Delete this Statistic Entry?')) return;
        try {
            await statisticApi.Delete(entry.StatisticEntryID);
            this.entries = this.entries.filter(current => current.StatisticEntryID !== entry.StatisticEntryID);
            if (this.nextOffset != null) this.nextOffset = Math.max(0, this.nextOffset - 1);
            this.render();
        } catch (error) {
            Popups.add_error(error instanceof Error ? error.message : 'Failed to delete Statistic Entry.');
        }
    }
}

if (!customElements.get('statistics-view')) {
    customElements.define('statistics-view', StatisticsView);
}

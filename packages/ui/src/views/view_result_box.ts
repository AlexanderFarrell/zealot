import { getNavigator } from '@websoil/engine';
import { ViewError } from '@zealot/api/src/view';
import type { Item } from '@zealot/domain/src/item';
import type { ViewDisplay, ViewResult } from '@zealot/domain/src/saved_view';
import { buildItemCardList } from './item_card_list';
import { ItemTableView } from './item_table_view';
import type { ItemTableColumn } from './item_table_types';


const DEFAULT_COLUMNS = ['Status', 'Priority'];
const PAGE_SIZE = 50;

export interface ViewBoxConfig {
    /** Runs the view for one page; the caller decides between a saved view and a draft definition. */
    load: (page: { limit: number; offset: number }) => Promise<ViewResult>;
    /** Shown while the title is not yet known, and as the link text when the result has none. */
    title: string;
    /** Item ID of the view, to link its title to its page. */
    viewId?: number;
    display: ViewDisplay;
    /** Present when the viewer may switch modes; absent hides the toggle. */
    onDisplayChange?: (display: ViewDisplay) => void;
    /** Present shows the Edit view button. */
    onEdit?: () => void;
    /** When true the box pages with Load more; otherwise it shows one page and "+N more". */
    paged?: boolean;
    /** Opens the view's own page; used by "+N more". */
    onMore?: () => void;
    /** Set when an error should offer a way to fix the view. */
    onFix?: () => void;
}

type Group = { label: string; items: Item[] };

/** Merges a later page's groups into the earlier ones, keyed by group key. */
function mergeGroups(into: Map<string, Group>, result: ViewResult): void {
    for (const group of result.Groups) {
        const key = JSON.stringify(group.key);
        const existing = into.get(key);
        if (existing) existing.items.push(...group.items);
        else into.set(key, { label: group.label, items: [...group.items] });
    }
}

/**
 * A live Saved View result in cards or table form. Used by `![[View]]` embeds and by the
 * `zealot-view` block on a Saved View's own page. Never shows item content, so a view
 * cannot recurse into itself.
 */
export class ViewResultBox extends HTMLElement {
    private _config: ViewBoxConfig | null = null;
    private _items: Item[] = [];
    private _groups = new Map<string, Group>();
    private _columns: string[] = [];
    private _total = 0;
    private _title = '';
    private _viewId: number | undefined;
    private _warnings: string[] = [];
    private _error: Error | null = null;
    private _loading = false;
    private _generation = 0;

    /** The item ID of the view once a run has resolved it. */
    get viewId(): number | undefined {
        return this._viewId;
    }

    init(config: ViewBoxConfig): this {
        this._config = config;
        this._title = config.title;
        this._viewId = config.viewId;
        this._render();
        void this.refresh();
        return this;
    }

    /** Changes the shown mode without refetching (the toggle already wrote the new mode). */
    setDisplay(display: ViewDisplay): void {
        if (!this._config || this._config.display === display) return;
        this._config.display = display;
        this._render();
    }

    /** Reloads the first page. */
    async refresh(): Promise<void> {
        await this._fetch(0);
    }

    private async _fetch(offset: number): Promise<void> {
        const config = this._config;
        if (!config) return;
        const generation = ++this._generation;
        this._loading = true;
        this._render();
        try {
            const result = await config.load({ limit: PAGE_SIZE, offset });
            if (generation !== this._generation) return;
            if (offset === 0) {
                this._items = [];
                this._groups = new Map();
            }
            this._items.push(...result.Items);
            mergeGroups(this._groups, result);
            this._columns = result.Columns;
            this._total = result.Total;
            // An unsaved run is titled by the server, so the caller's title wins.
            this._title = this._title || result.Title;
            this._viewId = result.ViewID ?? this._viewId;
            this._warnings = result.Warnings;
            this._error = null;
        } catch (error) {
            if (generation !== this._generation) return;
            this._error = error instanceof Error ? error : new Error(String(error));
        } finally {
            if (generation === this._generation) {
                this._loading = false;
                this._render();
            }
        }
    }

    private _render(): void {
        const config = this._config;
        if (!config) return;
        this.className = 'view-box';
        this.innerHTML = '';
        this.appendChild(this._buildHeader(config));

        if (this._error) {
            this.appendChild(this._buildError(config, this._error));
            return;
        }
        for (const warning of this._warnings) {
            const note = document.createElement('p');
            note.className = 'tool-muted view-box__warning';
            note.textContent = warning;
            this.appendChild(note);
        }
        if (this._loading && this._items.length === 0) {
            const note = document.createElement('p');
            note.className = 'tool-muted';
            note.textContent = 'Loading…';
            this.appendChild(note);
            return;
        }
        this.appendChild(config.display === 'table' ? this._buildTables() : this._buildCards());
        this.appendChild(this._buildFooter(config));
    }

    private _buildHeader(config: ViewBoxConfig): HTMLElement {
        const header = document.createElement('div');
        header.className = 'view-box__header';

        const linkId = this._viewId ?? (this._error instanceof ViewError ? this._error.itemId : undefined);
        const title = document.createElement(linkId === undefined ? 'span' : 'a');
        title.className = 'view-box__title';
        title.textContent = this._title;
        if (title instanceof HTMLAnchorElement && linkId !== undefined) {
            const id = linkId;
            title.href = `/item/${encodeURIComponent(this._title)}`;
            title.addEventListener('click', (event) => {
                event.preventDefault();
                getNavigator().openItemById(id);
            });
        }
        header.appendChild(title);

        if (!this._error && !(this._loading && this._items.length === 0)) {
            const total = document.createElement('span');
            total.className = 'view-box__total tool-muted';
            total.textContent = `${this._total} ${this._total === 1 ? 'item' : 'items'}`;
            header.appendChild(total);
        }

        const actions = document.createElement('div');
        actions.className = 'view-box__actions';

        if (config.onDisplayChange) {
            for (const mode of ['cards', 'table'] as const) {
                const button = document.createElement('button');
                button.type = 'button';
                button.className = 'view-box__mode';
                button.textContent = mode === 'cards' ? 'Cards' : 'Table';
                button.setAttribute('aria-pressed', String(config.display === mode));
                button.addEventListener('click', () => {
                    if (config.display === mode) return;
                    this.setDisplay(mode);
                    config.onDisplayChange?.(mode);
                });
                actions.appendChild(button);
            }
        }

        const refresh = document.createElement('button');
        refresh.type = 'button';
        refresh.className = 'view-box__refresh';
        refresh.title = 'Refresh';
        refresh.setAttribute('aria-label', 'Refresh');
        refresh.textContent = '↻';
        refresh.disabled = this._loading;
        refresh.addEventListener('click', () => { void this.refresh(); });
        actions.appendChild(refresh);

        if (config.onEdit) {
            const edit = document.createElement('button');
            edit.type = 'button';
            edit.className = 'view-box__edit';
            edit.textContent = 'Edit view';
            edit.addEventListener('click', () => config.onEdit?.());
            actions.appendChild(edit);
        }

        header.appendChild(actions);
        return header;
    }

    private _buildError(config: ViewBoxConfig, error: Error): HTMLElement {
        const wrap = document.createElement('div');
        wrap.className = 'view-box__error tool-error';
        const message = document.createElement('span');
        const notFound = error instanceof ViewError && error.kind === 'not_found';
        message.textContent = notFound ? 'Not a Saved View' : error.message;
        wrap.appendChild(message);
        if (config.onFix && !notFound) {
            const fix = document.createElement('button');
            fix.type = 'button';
            fix.textContent = 'Edit view';
            fix.addEventListener('click', () => config.onFix?.());
            wrap.append(' ', fix);
        }
        return wrap;
    }

    /** Server groups in order; a flat list when the view has none. */
    private _groupList(): Group[] {
        if (this._groups.size > 0) return [...this._groups.values()];
        return [{ label: '', items: this._items }];
    }

    private _buildCards(): HTMLElement {
        const hasGroups = this._groups.size > 0;
        return buildItemCardList(this._items, 'No items.', {
            showParent: true,
            dropZones: false,
            onDrop: () => { void this.refresh(); },
            ...(hasGroups ? { groups: this._groupList() } : {}),
        });
    }

    private _tableColumns(): ItemTableColumn[] {
        const keys = this._columns.length > 0 ? this._columns : DEFAULT_COLUMNS;
        const columns: ItemTableColumn[] = [{ kind: 'title', sortable: false }];
        for (const key of keys) {
            columns.push({ kind: 'attribute', attributeKey: key, sortable: false });
        }
        return columns;
    }

    private _buildTables(): HTMLElement {
        const wrap = document.createElement('div');
        wrap.className = 'view-box__tables';
        const columns = this._tableColumns();
        for (const group of this._groupList()) {
            if (group.label) {
                const label = document.createElement('div');
                label.className = 'item-card-group__label';
                label.textContent = group.label;
                wrap.appendChild(label);
            }
            const table = document.createElement('item-table-view') as ItemTableView;
            table.init({
                columns,
                items: group.items,
                emptyMessage: 'No items.',
                onChange: () => { void this.refresh(); },
            });
            wrap.appendChild(table);
        }
        return wrap;
    }

    private _buildFooter(config: ViewBoxConfig): HTMLElement {
        const footer = document.createElement('div');
        footer.className = 'view-box__footer';
        const remaining = this._total - this._items.length;
        if (remaining > 0) {
            const more = document.createElement('button');
            more.type = 'button';
            more.className = 'view-box__more';
            if (config.paged) {
                more.textContent = `Load more (${remaining})`;
                more.disabled = this._loading;
                more.addEventListener('click', () => { void this._fetch(this._items.length); });
            } else {
                more.textContent = `+${remaining} more`;
                more.addEventListener('click', () => config.onMore?.());
            }
            footer.appendChild(more);
        }
        return footer;
    }
}

if (!customElements.get('view-result-box')) customElements.define('view-result-box', ViewResultBox);

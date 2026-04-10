import type { ItemAPI } from '@zealot/api/src/item';
import { Events, ItemEvents, getNavigator, type AppLocation } from '@websoil/engine';
import { icons } from '@zealot/content';
import type { Item } from '@zealot/domain/src/item';

interface NavTreeToolViewOptions {
    itemApi: ItemAPI;
}

interface NavTreeNodeViewOptions {
    item: Item;
    itemApi: ItemAPI;
    isItemActive: (item: Item) => boolean;
}

class NavTreeNodeView extends HTMLElement {
    private item: Item | null = null;
    private itemApi: ItemAPI | null = null;
    private isItemActive: ((item: Item) => boolean) | null = null;
    private expanded = false;
    private childrenLoaded = false;
    private loadingChildren = false;
    private childItems: Item[] = [];
    private childError: string | null = null;
    private titleButton: HTMLButtonElement | null = null;

    init(options: NavTreeNodeViewOptions): this {
        this.item = options.item;
        this.itemApi = options.itemApi;
        this.isItemActive = options.isItemActive;
        if (this.isConnected) {
            this.render();
        }
        return this;
    }

    connectedCallback(): void {
        this.render();
    }

    refreshActiveState(): void {
        if (!this.item || !this.titleButton || !this.isItemActive) {
            return;
        }
        this.titleButton.classList.toggle('is-active', this.isItemActive(this.item));
    }

    private render(): void {
        if (!this.item || !this.itemApi || !this.isItemActive) {
            return;
        }

        this.innerHTML = `
        <div class="nav-tree-node ${this.expanded ? 'is-expanded' : ''}">
            <div class="nav-tree-row">
                <button type="button" class="nav-tree-toggle" aria-label="Toggle children" aria-expanded="${this.expanded}">
                    <img class="nav-tree-chevron" src="${icons.right}" alt="">
                </button>
                <button type="button" class="nav-tree-title">${this.item.DisplayTitle}</button>
            </div>
            <div class="nav-tree-children"></div>
        </div>
        `;

        const toggleButton = this.querySelector('.nav-tree-toggle');
        const childrenEl = this.querySelector('.nav-tree-children');
        this.titleButton = this.querySelector('.nav-tree-title');

        toggleButton?.addEventListener('click', () => {
            void this.toggleExpanded();
        });

        this.titleButton?.addEventListener('click', () => {
            getNavigator().openItemById(this.item!.ItemID);
        });

        this.refreshActiveState();

        if (!childrenEl || !this.expanded) {
            return;
        }

        if (this.loadingChildren) {
            childrenEl.innerHTML = `<p class="tool-muted">Loading children…</p>`;
            return;
        }

        if (this.childError) {
            childrenEl.innerHTML = `<p class="tool-error">${this.childError}</p>`;
            return;
        }

        if (this.childrenLoaded && this.childItems.length === 0) {
            childrenEl.innerHTML = `<p class="tool-muted">No child items.</p>`;
            return;
        }

        this.childItems.forEach((child) => {
            childrenEl.appendChild(new NavTreeNodeView().init({
                item: child,
                itemApi: this.itemApi!,
                isItemActive: this.isItemActive!,
            }));
        });
    }

    private async toggleExpanded(): Promise<void> {
        this.expanded = !this.expanded;
        if (this.expanded && !this.childrenLoaded && !this.loadingChildren) {
            await this.loadChildren();
            return;
        }
        this.render();
    }

    private async loadChildren(): Promise<void> {
        this.loadingChildren = true;
        this.childError = null;
        this.render();

        try {
            this.childItems = await this.itemApi!.GetChildren(this.item!.ItemID);
            this.childrenLoaded = true;
        } catch (error) {
            console.error(error);
            this.childError = (error as Error).message ?? 'Failed to load child items.';
        } finally {
            this.loadingChildren = false;
            this.render();
        }
    }
}

export class NavTreeToolView extends HTMLElement {
    private itemApi: ItemAPI | null = null;
    private bodyEl: HTMLDivElement | null = null;
    private location: AppLocation = { kind: 'not_found', path: '' };
    private unsubscribeLocation: (() => void) | null = null;
    private loaded = false;
    private loading = false;
    private loadError: string | null = null;
    private rootItems: Item[] = [];
    private readonly handleLocationChange = (location: AppLocation) => {
        this.location = location;
        this.refreshActiveState();
    };
    private readonly handleItemMutation = () => {
        void this.reload();
    };

    init(options: NavTreeToolViewOptions): this {
        this.itemApi = options.itemApi;
        if (this.isConnected) {
            this.render();
            this.attachSubscriptions();
            if (!this.loaded && !this.loading) {
                void this.loadRoots();
            }
        }
        return this;
    }

    connectedCallback(): void {
        this.render();
        if (!this.itemApi) {
            return;
        }
        this.attachSubscriptions();
        if (!this.loaded && !this.loading) {
            void this.loadRoots();
        } else {
            this.refreshActiveState();
        }
    }

    disconnectedCallback(): void {
        this.detachSubscriptions();
    }

    private render(): void {
        this.innerHTML = `
        <div class="tool-panel">
            <div class="tool-panel-header">
                <h2>Navigation</h2>
            </div>
            <div class="nav-tree-tool-body"></div>
        </div>
        `;
        this.bodyEl = this.querySelector('.nav-tree-tool-body');
        this.renderBody();
    }

    private attachSubscriptions(): void {
        const navigator = getNavigator();
        this.location = navigator.getLocation();
        this.unsubscribeLocation?.();
        this.unsubscribeLocation = navigator.subscribe(this.handleLocationChange);
        Events.off(ItemEvents.created, this.handleItemMutation);
        Events.off(ItemEvents.deleted, this.handleItemMutation);
        Events.on(ItemEvents.created, this.handleItemMutation);
        Events.on(ItemEvents.deleted, this.handleItemMutation);
    }

    private detachSubscriptions(): void {
        this.unsubscribeLocation?.();
        this.unsubscribeLocation = null;
        Events.off(ItemEvents.created, this.handleItemMutation);
        Events.off(ItemEvents.deleted, this.handleItemMutation);
    }

    private async loadRoots(): Promise<void> {
        this.loading = true;
        this.loadError = null;
        this.renderBody();

        try {
            this.rootItems = await this.itemApi!.GetAll();
            this.loaded = true;
        } catch (error) {
            console.error(error);
            this.loadError = (error as Error).message ?? 'Failed to load root items.';
        } finally {
            this.loading = false;
            this.renderBody();
            this.refreshActiveState();
        }
    }

    private async reload(): Promise<void> {
        this.loaded = false;
        this.rootItems = [];
        await this.loadRoots();
    }

    private renderBody(): void {
        if (!this.bodyEl) {
            return;
        }

        this.bodyEl.innerHTML = '';

        if (this.loading) {
            this.bodyEl.innerHTML = `<p class="tool-muted">Loading navigation tree…</p>`;
            return;
        }

        if (this.loadError) {
            this.bodyEl.innerHTML = `<p class="tool-error">${this.loadError}</p>`;
            return;
        }

        if (this.loaded && this.rootItems.length === 0) {
            this.bodyEl.innerHTML = `<p class="tool-muted">Set the Root attribute to true on items to display them here.</p>`;
            return;
        }

        this.rootItems.forEach((item) => {
            this.bodyEl?.appendChild(new NavTreeNodeView().init({
                item,
                itemApi: this.itemApi!,
                isItemActive: (candidate) => this.isItemActive(candidate),
            }));
        });
    }

    private isItemActive(item: Item): boolean {
        if (this.location.kind !== 'item') {
            return false;
        }

        if (this.location.itemId !== undefined) {
            return item.ItemID === this.location.itemId;
        }

        if (this.location.title !== undefined) {
            return item.Title === this.location.title;
        }

        return false;
    }

    private refreshActiveState(): void {
        this.querySelectorAll('nav-tree-node-view').forEach((node) => {
            (node as unknown as NavTreeNodeView).refreshActiveState();
        });
    }
}

if (!customElements.get('nav-tree-node-view')) {
    customElements.define('nav-tree-node-view', NavTreeNodeView);
}

if (!customElements.get('nav-tree-tool-view')) {
    customElements.define('nav-tree-tool-view', NavTreeToolView);
}

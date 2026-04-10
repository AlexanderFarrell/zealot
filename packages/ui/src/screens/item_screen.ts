import { BaseElementEmpty, Popups, getNavigator } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';
import { icons } from '@zealot/content';
import { AttributeEditor } from '../views/attribute_editor';
import { ItemTableView, type ItemTableColumn } from '../views/item_table_view';

const itemApi = new ItemAPI('/api');
const collectionColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Status', kind: 'attribute', label: 'Status' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
];

let content_visible = true;
let related_visible = true;

export class ItemScreen extends BaseElementEmpty {
    private item: Item | null = null;
    private last_loaded_title: string | null = null;
    private content_debounce: ReturnType<typeof setTimeout> | null = null;

    async render() {
        // Called by BaseElementEmpty.connectedCallback — nothing to do until
        // loadItem / loadItemById is called.
    }

    loadItem(title: string): void {
        this.last_loaded_title = title;
        void this.fetchAndRender(() => itemApi.GetByTitle(title));
    }

    loadItemById(id: number): void {
        this.last_loaded_title = null;
        void this.fetchAndRender(() => itemApi.GetById(id));
    }

    private async fetchAndRender(fetch: () => Promise<Item>): Promise<void> {
        this.innerHTML = '';
        this.appendChild(new LoadingSpinner());

        try {
            this.item = await fetch();
        } catch {
            this.item = null;
        }

        this.innerHTML = '';

        if (this.item == null) {
            this.renderNotFound();
            return;
        }

        this.renderItem();
    }

    private renderNotFound(): void {
        const msg = document.createElement('p');
        msg.textContent = "That item doesn't exist.";
        this.appendChild(msg);

        if (this.last_loaded_title) {
            const btn = document.createElement('button');
            btn.textContent = 'Create it?';
            btn.addEventListener('click', async () => {
                try {
                    await itemApi.Add({ title: this.last_loaded_title!, content: '' });
                    this.loadItem(this.last_loaded_title!);
                } catch (e) {
                    Popups.add_error((e as Error).message ?? 'Failed to create item.');
                }
            });
            this.appendChild(btn);
            btn.focus();
        }
    }

    private renderItem(): void {
        const item = this.item!;

        // Action buttons
        this.appendChild(this.buildActions(item));

        // Title
        const title = document.createElement('h1');
        title.contentEditable = 'true';
        title.innerText = item.Title;
        title.addEventListener('input', () => {
            item.Title = title.textContent ?? item.Title;
        });
        title.addEventListener('blur', () => {
            void itemApi.Update(item.ItemID, { item_id: item.ItemID, title: item.Title });
        });
        this.appendChild(title);

        // Types
        const typesDiv = document.createElement('div');
        typesDiv.className = 'item-types';
        this.appendChild(typesDiv);
        this.renderTypes(item, typesDiv);

        // Attributes
        const attrsSection = document.createElement('section');
        attrsSection.className = 'item-attributes';
        this.appendChild(attrsSection);
        this.renderAttributes(item, attrsSection);

        // Content
        const contentSection = document.createElement('section');
        contentSection.className = 'item-content';
        contentSection.style.display = content_visible ? 'block' : 'none';
        this.appendChild(contentSection);
        this.renderContent(item, contentSection);

        // Children + Related collections
        const relatedSection = document.createElement('section');
        relatedSection.className = 'item-collections';
        relatedSection.style.display = related_visible ? 'block' : 'none';
        this.appendChild(relatedSection);
        void this.renderCollections(item, relatedSection);
    }

    private buildActions(item: Item): HTMLElement {
        const row = document.createElement('div');
        row.className = 'button_row row gap';

        const makeBtn = (iconUrl: string, label: string, onClick: () => void): HTMLButtonElement => {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.title = label;
            const img = document.createElement('img');
            img.src = iconUrl;
            img.alt = label;
            img.style.width = '1.2em';
            img.style.height = '1.2em';
            btn.appendChild(img);
            btn.addEventListener('click', onClick);
            return btn;
        };

        row.appendChild(makeBtn(icons.up, 'To Parent', () => {
            const parentId = this.getParentItemId(item);
            if (parentId != null) {
                getNavigator().openItemById(parentId);
            } else {
                getNavigator().openHome();
            }
        }));

        row.appendChild(makeBtn(icons.link, 'Copy Link', () => {
            void navigator.clipboard.writeText(window.location.href);
            Popups.add('Link copied');
        }));

        row.appendChild(makeBtn(icons.expand, 'Open in New Tab', () => {
            window.open(window.location.href, '_blank');
        }));

        row.appendChild(makeBtn(icons.edit, 'Toggle Content', () => {
            content_visible = !content_visible;
            const section = this.querySelector('.item-content') as HTMLElement | null;
            if (section) section.style.display = content_visible ? 'block' : 'none';
        }));

        row.appendChild(makeBtn(icons.items, 'Toggle Related', () => {
            related_visible = !related_visible;
            const section = this.querySelector('.item-collections') as HTMLElement | null;
            if (section) section.style.display = related_visible ? 'block' : 'none';
        }));

        row.appendChild(makeBtn(icons.delete, 'Delete Item', () => {
            void (async () => {
                const confirmed = await ConfirmDialog.show('Are you sure you want to delete this item?');
                if (!confirmed) return;
                await itemApi.Delete(item.ItemID);
                Popups.add(`Removed ${item.Title}`);
                const parentId = this.getParentItemId(item);
                if (parentId != null) {
                    getNavigator().openItemById(parentId);
                } else {
                    getNavigator().openHome();
                }
            })();
        }));

        return row;
    }

    private renderTypes(item: Item, container: HTMLElement): void {
        container.innerHTML = '';
        if (item.Types.length === 0) return;

        item.Types.forEach((typeRef) => {
            const badge = document.createElement('span');
            badge.className = 'tool-badge';
            badge.textContent = typeRef.Name;
            badge.style.cursor = 'pointer';
            badge.addEventListener('click', () => {
                getNavigator().openType(typeRef.Name);
            });
            container.appendChild(badge);
        });
    }

    private renderAttributes(item: Item, container: HTMLElement): void {
        container.innerHTML = '';
        const editor = new AttributeEditor();
        container.appendChild(editor);
        editor.init(item);
    }

    private renderContent(item: Item, container: HTMLElement): void {
        const textarea = document.createElement('textarea');
        textarea.value = item.Content;
        textarea.style.width = '100%';
        textarea.style.minHeight = '200px';
        textarea.placeholder = 'Content…';
        textarea.addEventListener('input', () => {
            item.Content = textarea.value;
            if (this.content_debounce) clearTimeout(this.content_debounce);
            this.content_debounce = setTimeout(async () => {
                await itemApi.Update(item.ItemID, { item_id: item.ItemID, content: item.Content });
                Popups.add('Saved', 'note', 2);
            }, 1000);
        });
        container.appendChild(textarea);
    }

    private async renderCollections(item: Item, container: HTMLElement): Promise<void> {
        container.innerHTML = '';

        const children = this.buildCollectionSection('Children');
        const related = this.buildCollectionSection('Related');

        container.appendChild(children.section);
        container.appendChild(related.section);

        await Promise.all([
            this.renderCollectionTable({
                container: children.content,
                createRow: {
                    contextItemId: item.ItemID,
                    enabled: true,
                    relationship: 'parent',
                    submitLabel: 'Add Child',
                },
                emptyMessage: 'No child items.',
                errorMessage: 'Failed to load child items.',
                loader: () => itemApi.GetChildren(item.ItemID),
            }),
            this.renderCollectionTable({
                container: related.content,
                emptyMessage: 'No related items.',
                errorMessage: 'Failed to load related items.',
                loader: () => itemApi.GetRelated(item.ItemID),
            }),
        ]);
    }

    private buildCollectionSection(title: string): { content: HTMLElement; section: HTMLElement } {
        const section = document.createElement('section');
        section.className = 'item-collection-section';

        const heading = document.createElement('h2');
        heading.textContent = title;
        section.appendChild(heading);

        const content = document.createElement('div');
        content.className = 'item-collection-content';
        section.appendChild(content);

        return { content, section };
    }

    private async renderCollectionTable(args: {
        container: HTMLElement;
        createRow?: {
            contextItemId: number;
            enabled: boolean;
            relationship: 'parent';
            submitLabel: string;
        };
        emptyMessage: string;
        errorMessage: string;
        loader: () => Promise<Item[]>;
    }): Promise<void> {
        args.container.innerHTML = '';
        args.container.appendChild(new LoadingSpinner());

        let items: Item[];
        try {
            items = await args.loader();
        } catch {
            args.container.innerHTML = `<p>${args.errorMessage}</p>`;
            return;
        }

        args.container.innerHTML = '';
        const table = new ItemTableView();
        args.container.appendChild(table);
        const config = {
            columns: collectionColumns,
            emptyMessage: args.emptyMessage,
            items,
        };
        if (args.createRow) {
            Object.assign(config, { createRow: args.createRow });
        }
        table.init(config);
    }

    private getParentItemId(item: Item): number | null {
        const parentLink = item.Links.find((link) => link.Relationship === 'parent');
        return parentLink?.OtherItemID ?? null;
    }
}

customElements.define('item-screen', ItemScreen);

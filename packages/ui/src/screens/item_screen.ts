import { BaseElementEmpty, Popups, commands, getNavigator, openInNewTab, getRightSidebarHost, registerContextMenu, unregisterContextMenu, unregisterContextMenuIn } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';
import { icons } from '@zealot/content';
import { AttributeEditor } from '../views/attribute_editor';
import { CommentsView } from '../views/comments_view';
import { ZealotScriptEditor } from '../zealotscript/zealotscript_editor';
import { buildItemCardList } from '../views/item_card_list';
import { buildAddPanel } from '../views/item_table_add_panel';
import { createItem } from '../views/item_table_save';
import { loadAttributeKinds } from '../views/attribute_value_input';
import type { CreateDraftState } from '../views/item_table_types';
import { AssignTypeModal } from '../common/assign_type_modal';
import { PasteTemplateModal } from '../common/paste_template_modal';
import { createItemTitleIconElement } from '../views/item_title';

const itemApi = new ItemAPI('/api');

let content_visible = true;

const VIEW_MODE_KEY = 'zealot:view-mode';

type ViewMode = 'narrow' | 'reader' | 'wide' | 'full';
const VIEW_MODE_CYCLE: ViewMode[] = ['narrow', 'reader', 'wide', 'full'];

interface ViewModeConfig {
    icon: string;
    title: string;
    cssClass: string;
}

function getViewModeConfig(icons: Record<string, string>): Record<ViewMode, ViewModeConfig> {
    return {
        narrow:  { icon: icons['center']!,   title: 'Narrow View',  cssClass: 'item-screen--narrow'  },
        reader:  { icon: icons['pageview']!,  title: 'Reader View',  cssClass: 'item-screen--reader'  },
        wide:    { icon: icons['monitor']!,   title: 'Wide View',    cssClass: 'item-screen--wide'    },
        full:    { icon: icons['expand']!,    title: 'Full Width',   cssClass: 'item-screen--full'    },
    };
}

function getViewMode(): ViewMode {
    const stored = localStorage.getItem(VIEW_MODE_KEY);
    if (stored === 'narrow' || stored === 'reader' || stored === 'wide' || stored === 'full') return stored;
    return 'full';
}

function applyViewMode(el: HTMLElement, mode: ViewMode, configs: Record<ViewMode, ViewModeConfig>): void {
    localStorage.setItem(VIEW_MODE_KEY, mode);
    for (const cfg of Object.values(configs)) {
        el.classList.remove(cfg.cssClass);
    }
    el.classList.add(configs[mode].cssClass);
}

function nextViewMode(current: ViewMode): ViewMode {
    const idx = VIEW_MODE_CYCLE.indexOf(current);
    return VIEW_MODE_CYCLE[(idx + 1) % VIEW_MODE_CYCLE.length]!;
}

const ITEM_CONTEXT_COMMANDS = [
    'Item: Go to Parent',
    'Item: Copy Link',
    'Item: Open in New Tab',
    'Item: Toggle Content',
    'Item: Delete',
    'Item: Copy as Markdown',
    'Item: Download PDF',
    'Item: Download DOCX',
    'Item: Manage Types',
    'Item: Paste Template',
] as const;

export class ItemScreen extends BaseElementEmpty {
    private item: Item | null = null;
    private last_loaded_title: string | null = null;
    private content_debounce: ReturnType<typeof setTimeout> | null = null;
    private _titleListener: ((title: string) => void) | null = null;
    private _sidebarEl: HTMLElement | null = null;

    setTitleListener(fn: (title: string) => void): void {
        this._titleListener = fn;
    }

    onActivated(): void {
        getRightSidebarHost()?.setContent(this._sidebarEl);
    }

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

    disconnectedCallback(): void {
        getRightSidebarHost()?.setContent(null);
        unregisterContextMenu(this);
        for (const name of ITEM_CONTEXT_COMMANDS) {
            commands.runner.Commands.delete(name);
        }
    }

    private async fetchAndRender(fetch: () => Promise<Item>): Promise<void> {
        unregisterContextMenu(this);
        this._sidebarEl = null;
        getRightSidebarHost()?.setContent(null);
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
        // Apply persisted view mode
        const viewConfigs = getViewModeConfig(icons);
        applyViewMode(this, getViewMode(), viewConfigs);

        // Types
        const typesDiv = document.createElement('div');
        typesDiv.className = 'item-types';

        // Attributes
        const attrsSection = document.createElement('section');
        attrsSection.className = 'item-attributes';

        const titleRow = document.createElement('div');
        titleRow.className = 'item-title-row';
        const iconEl = document.createElement('span');
        iconEl.className = 'item-title-icon';
        iconEl.draggable = true;
        iconEl.addEventListener('dragstart', (event) => {
            if (iconEl.hidden) {
                event.preventDefault();
                return;
            }
            event.dataTransfer?.setData('text/plain', String(item.ItemID));
            if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
        });
        const renderTitleIcon = () => {
            const icon = createItemTitleIconElement(item, { className: 'item-screen-title-icon' });
            iconEl.replaceChildren(...(icon ? [icon] : []));
            iconEl.hidden = !icon;
        };

        const onTypesDone = () => {
            this.renderTypes(item, typesDiv, onTypesDone);
            this.renderAttributes(item, attrsSection, renderTitleIcon);
        };

        registerContextMenu(this, () => [
            { label: 'Copy Link', onClick: () => {
                void navigator.clipboard.writeText(window.location.href);
                Popups.add('Link copied');
            }},
            { label: 'Open in New Tab', onClick: () => openInNewTab(window.location.pathname) },
            { label: 'Open in New Window', onClick: () => window.open(window.location.href, '_blank', 'noopener,noreferrer') },
            { label: 'Copy as Markdown', onClick: () => {
                void navigator.clipboard.writeText(this.buildMarkdown(item));
                Popups.add('Copied as Markdown');
            }},
            { label: 'Download as PDF', onClick: () => { window.location.href = itemApi.ExportPdfUrl(item.ItemID); } },
            { label: 'Download as DOCX', onClick: () => { window.location.href = itemApi.ExportDocxUrl(item.ItemID); } },
            { separator: true },
            { label: 'Manage Types', onClick: () => AssignTypeModal.show(item, () => onTypesDone()) },
            { label: 'Paste Template', onClick: () => {
                PasteTemplateModal.show((templateContent) => {
                    const separator = item.Content.trim().length > 0 ? '\n\n' : '';
                    item.Content = item.Content + separator + templateContent;
                    const editorEl = this.querySelector('zealotscript-editor') as ZealotScriptEditor | null;
                    if (editorEl) editorEl.content = item.Content;
                    void itemApi.Update(item.ItemID, { item_id: item.ItemID, content: item.Content })
                        .then(() => Popups.add('Template pasted'));
                });
            }},
            { separator: true },
            { label: 'Delete Item', danger: true, onClick: () => {
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
            }},
        ]);

        // Action buttons
        this.appendChild(this.buildActions(item, () => onTypesDone()));

        // Icon + Title row
        renderTitleIcon();
        titleRow.appendChild(iconEl);

        // Title
        const title = document.createElement('h1');
        title.contentEditable = 'true';
        title.innerText = item.Title;
        title.addEventListener('input', () => {
            item.Title = title.textContent ?? item.Title;
        });
        title.addEventListener('blur', () => {
            void itemApi.Update(item.ItemID, { item_id: item.ItemID, title: item.Title }).then((updated) => {
                // Keep the URL in sync so a reload fetches the item by its current title.
                if (this.last_loaded_title != null && updated.Title !== this.last_loaded_title) {
                    window.history.replaceState(null, '', `/item/${encodeURIComponent(updated.Title)}`);
                    this.last_loaded_title = updated.Title;
                }
            });
        });
        titleRow.appendChild(title);
        this.appendChild(titleRow);

        this.appendChild(typesDiv);
        this.appendChild(attrsSection);
        this.renderTypes(item, typesDiv, onTypesDone);
        this.renderAttributes(item, attrsSection, renderTitleIcon);

        // Content
        const contentSection = document.createElement('section');
        contentSection.className = 'item-content';
        contentSection.style.display = content_visible ? 'block' : 'none';
        this.appendChild(contentSection);
        this.renderContent(item, contentSection);

        // Children + Related collections — rendered into the right sidebar
        const collectionsEl = document.createElement('section');
        collectionsEl.className = 'item-collections';
        this._sidebarEl = collectionsEl;
        this._titleListener?.(item.Title);
        getRightSidebarHost()?.setContent(collectionsEl);
        void this.renderCollections(item, collectionsEl);

        const commentsSection = this.buildCollectionSection('Comments');
        const commentsView = new CommentsView().init({
            scope: { kind: 'item', itemId: item.ItemID },
        });
        commentsSection.content.appendChild(commentsView);
        this.appendChild(commentsSection.section);

        this.registerItemContextCommands(item, onTypesDone);
    }

    private registerItemContextCommands(item: Item, onTypesDone: () => void): void {
        const goToParent = () => {
            const parentId = this.getParentItemId(item);
            if (parentId != null) getNavigator().openItemById(parentId);
            else getNavigator().openHome();
        };
        commands.runner.register('Item: Go to Parent', [], goToParent);
        commands.runner.register('Item: Copy Link', [], () => {
            void navigator.clipboard.writeText(window.location.href);
            Popups.add('Link copied');
        });
        commands.runner.register('Item: Open in New Tab', [], () => openInNewTab(window.location.pathname));
        commands.runner.register('Item: Toggle Content', [], () => {
            content_visible = !content_visible;
            const section = this.querySelector('.item-content') as HTMLElement | null;
            if (section) section.style.display = content_visible ? 'block' : 'none';
        });
        commands.runner.register('Item: Delete', [], () => {
            void (async () => {
                const confirmed = await ConfirmDialog.show('Are you sure you want to delete this item?');
                if (!confirmed) return;
                await itemApi.Delete(item.ItemID);
                Popups.add(`Removed ${item.Title}`);
                const parentId = this.getParentItemId(item);
                if (parentId != null) getNavigator().openItemById(parentId);
                else getNavigator().openHome();
            })();
        });
        commands.runner.register('Item: Copy as Markdown', [], () => {
            void navigator.clipboard.writeText(this.buildMarkdown(item));
            Popups.add('Copied as Markdown');
        });
        commands.runner.register('Item: Download PDF', [], () => { window.location.href = itemApi.ExportPdfUrl(item.ItemID); });
        commands.runner.register('Item: Download DOCX', [], () => { window.location.href = itemApi.ExportDocxUrl(item.ItemID); });
        commands.runner.register('Item: Manage Types', [], () => AssignTypeModal.show(item, onTypesDone));
        commands.runner.register('Item: Paste Template', [], () => {
            PasteTemplateModal.show((templateContent) => {
                const separator = item.Content.trim().length > 0 ? '\n\n' : '';
                item.Content = item.Content + separator + templateContent;
                const editorEl = this.querySelector('zealotscript-editor') as ZealotScriptEditor | null;
                if (editorEl) editorEl.content = item.Content;
                void itemApi.Update(item.ItemID, { item_id: item.ItemID, content: item.Content })
                    .then(() => Popups.add('Template pasted'));
            });
        });
    }

    private buildActions(item: Item, onManageTypes: () => void): HTMLElement {
        const row = document.createElement('div');
        row.className = 'button_row row gap';

        const makeBtn = (iconUrl: string, label: string, onClick: () => void): HTMLButtonElement => {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.title = label;
            btn.className = 'item-action-btn';
            const img = document.createElement('img');
            img.src = iconUrl;
            img.alt = label;
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
            openInNewTab(window.location.pathname);
        }));

        row.appendChild(makeBtn(icons.edit, 'Toggle Content', () => {
            content_visible = !content_visible;
            const section = this.querySelector('.item-content') as HTMLElement | null;
            if (section) section.style.display = content_visible ? 'block' : 'none';
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

        row.appendChild(this.buildDownloadBtn(item));

        row.appendChild(makeBtn(icons.tag, 'Manage types', () => {
            AssignTypeModal.show(item, onManageTypes);
        }));

        row.appendChild(makeBtn(icons.postAdd, 'Paste Template', () => {
            PasteTemplateModal.show((templateContent) => {
                const separator = item.Content.trim().length > 0 ? '\n\n' : '';
                item.Content = item.Content + separator + templateContent;
                const editorEl = this.querySelector('zealotscript-editor') as ZealotScriptEditor | null;
                if (editorEl) editorEl.content = item.Content;
                void itemApi.Update(item.ItemID, { item_id: item.ItemID, content: item.Content })
                    .then(() => Popups.add('Template pasted'));
            });
        }));

        row.appendChild(makeBtn(icons.schedule, 'Time Blocks', () => {
            getNavigator().openTimeBlocksForItem(item.ItemID);
        }));

        const viewConfigs = getViewModeConfig(icons);
        const viewBtnImg = document.createElement('img');
        const updateViewBtn = (mode: ViewMode) => {
            const cfg = viewConfigs[mode];
            viewBtnImg.src = cfg.icon;
            viewBtnImg.alt = cfg.title;
            viewBtn.title = cfg.title;
        };
        const viewBtn = document.createElement('button');
        viewBtn.type = 'button';
        viewBtn.className = 'item-action-btn item-action-btn--view-toggle';
        viewBtn.appendChild(viewBtnImg);
        updateViewBtn(getViewMode());
        viewBtn.addEventListener('click', () => {
            const next = nextViewMode(getViewMode());
            applyViewMode(this, next, viewConfigs);
            updateViewBtn(next);
        });
        row.appendChild(viewBtn);

        return row;
    }

    private buildDownloadBtn(item: Item): HTMLElement {
        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'position: relative; display: inline-block;';

        const btn = document.createElement('button');
        btn.type = 'button';
        btn.title = 'Download / Export';
        btn.className = 'item-action-btn';
        const img = document.createElement('img');
        img.src = icons.download;
        img.alt = 'Download';
        btn.appendChild(img);

        const menu = document.createElement('div');
        menu.style.cssText = [
            'display: none',
            'position: absolute',
            'top: calc(100% + 4px)',
            'left: 0',
            'background: var(--panel-0, var(--bg-2))',
            'border: 1px solid var(--border-0, var(--border-1))',
            'border-radius: 4px',
            'box-shadow: 0 2px 8px rgba(0,0,0,.2)',
            'min-width: 170px',
            'z-index: 200',
        ].join(';');

        menu.appendChild(this.makeMenuItem('Copy Markdown', () => {
            const md = this.buildMarkdown(item);
            void navigator.clipboard.writeText(md);
            Popups.add('Copied as Markdown');
            menu.style.display = 'none';
        }));

        menu.appendChild(this.makeMenuItem('Download as PDF', () => {
            window.location.href = itemApi.ExportPdfUrl(item.ItemID);
            menu.style.display = 'none';
        }));

        menu.appendChild(this.makeMenuItem('Download as DOCX', () => {
            window.location.href = itemApi.ExportDocxUrl(item.ItemID);
            menu.style.display = 'none';
        }));

        wrapper.appendChild(btn);
        wrapper.appendChild(menu);

        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
        });

        document.addEventListener('click', () => {
            menu.style.display = 'none';
        }, { capture: false });

        return wrapper;
    }

    private makeMenuItem(label: string, onClick: () => void): HTMLButtonElement {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.textContent = label;
        btn.style.cssText = [
            'display: block',
            'width: 100%',
            'text-align: left',
            'padding: 6px 12px',
            'background: none',
            'border: none',
            'cursor: pointer',
            'font-size: 0.9em',
            'white-space: nowrap',
            'color: inherit',
        ].join(';');
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            onClick();
        });
        return btn;
    }

    private buildMarkdown(item: Item): string {
        const lines: string[] = [];
        lines.push(`# ${item.Title}`, '');

        const attrs = Object.entries(item.Attributes);
        if (attrs.length > 0) {
            for (const [k, v] of attrs) {
                lines.push(`**${k}**: ${String(v)}`);
            }
            lines.push('');
        }

        lines.push(item.Content);
        return lines.join('\n');
    }

    private renderTypes(item: Item, container: HTMLElement, _onDone: () => void): void {
        unregisterContextMenuIn(container);
        container.innerHTML = '';

        item.Types.forEach((typeRef) => {
            const badge = document.createElement('span');
            badge.className = 'tool-badge';
            badge.textContent = typeRef.Name;
            badge.style.cursor = 'pointer';
            badge.addEventListener('click', () => {
                getNavigator().openType(typeRef.Name);
            });
            registerContextMenu(badge, () => [
                { label: 'Open Type Screen', onClick: () => getNavigator().openType(typeRef.Name) },
                { separator: true },
                { label: 'Remove from Item', danger: true, onClick: () => {
                    void (async () => {
                        await itemApi.UnassignType(item.ItemID, typeRef.Name);
                        Popups.add(`Removed type "${typeRef.Name}"`);
                        _onDone();
                    })();
                }},
            ]);
            container.appendChild(badge);
        });
    }

    private renderAttributes(item: Item, container: HTMLElement, onIconChange?: () => void): void {
        container.innerHTML = '';
        const editor = new AttributeEditor();
        editor.onIconChange = onIconChange ?? null;
        container.appendChild(editor);
        editor.init(item);
    }

    private renderContent(item: Item, container: HTMLElement): void {
        const editor = document.createElement('zealotscript-editor') as ZealotScriptEditor;
        editor.content = item.Content;
        editor.addEventListener('change', (e: Event) => {
            const value = (e as CustomEvent<string>).detail;
            item.Content = value;
            if (this.content_debounce) clearTimeout(this.content_debounce);
            this.content_debounce = setTimeout(async () => {
                await itemApi.Update(item.ItemID, { item_id: item.ItemID, content: item.Content });
                Popups.add('Saved', 'note', 2);
            }, 1000);
        });
        container.appendChild(editor);
    }

    private async renderCollections(item: Item, container: HTMLElement): Promise<void> {
        container.innerHTML = '';

        const children = this.buildCollectionSection('Children');
        const related = this.buildCollectionSection('Related');

        // Backlinks section — lazy-loaded on first expand
        const backlinksSection = document.createElement('section');
        backlinksSection.className = 'item-collection-section';

        const backlinksToggle = document.createElement('button');
        backlinksToggle.className = 'item-collection-toggle';
        backlinksToggle.textContent = 'Backlinks ▶';
        backlinksSection.appendChild(backlinksToggle);

        const backlinksContent = document.createElement('div');
        backlinksContent.className = 'item-collection-content item-collection-content--hidden';
        backlinksSection.appendChild(backlinksContent);

        let backlinksLoaded = false;
        backlinksToggle.addEventListener('click', () => {
            const nowHidden = backlinksContent.classList.toggle('item-collection-content--hidden');
            backlinksToggle.textContent = `Backlinks ${nowHidden ? '▶' : '▼'}`;
            if (!nowHidden && !backlinksLoaded) {
                backlinksLoaded = true;
                void this.renderCollectionCards({
                    container: backlinksContent,
                    emptyMessage: 'No backlinks.',
                    errorMessage: 'Failed to load backlinks.',
                    grouped: true,
                    showParent: true,
                    loader: () => itemApi.GetBacklinks(item.ItemID),
                });
            }
        });

        container.appendChild(children.section);
        container.appendChild(related.section);
        container.appendChild(backlinksSection);

        await Promise.all([
            this.renderCollectionCards({
                container: children.content,
                createRow: { contextItemId: item.ItemID, relationship: 'parent', submitLabel: 'Add child' },
                emptyMessage: 'No child items.',
                errorMessage: 'Failed to load child items.',
                grouped: true,
                loader: () => itemApi.GetChildren(item.ItemID),
            }),
            this.renderCollectionCards({
                container: related.content,
                emptyMessage: 'No related items.',
                errorMessage: 'Failed to load related items.',
                grouped: true,
                showParent: true,
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

    private async renderCollectionCards(args: {
        container: HTMLElement;
        createRow?: { contextItemId: number; relationship: 'parent'; submitLabel: string };
        emptyMessage: string;
        errorMessage: string;
        grouped?: boolean;
        showParent?: boolean;
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

        let draft: CreateDraftState = { attributes: {}, title: '', types: [] };
        let panelOpen = false;
        let createError: string | null = null;
        let attributeKinds: Awaited<ReturnType<typeof loadAttributeKinds>> = {};

        const render = (): void => {
            unregisterContextMenuIn(args.container);
            args.container.innerHTML = '';

            if (createError) {
                const err = document.createElement('p');
                err.className = 'tool-error';
                err.textContent = createError;
                args.container.appendChild(err);
            }

            args.container.appendChild(buildItemCardList(items, args.emptyMessage, { ...(args.grouped ? { grouped: true } : {}), ...(args.showParent ? { showParent: true } : {}), onDrop: render }));

            if (args.createRow) {
                args.container.appendChild(buildAddPanel(
                    { contextItemId: args.createRow.contextItemId, enabled: true, panelMode: true, relationship: args.createRow.relationship, submitLabel: args.createRow.submitLabel },
                    draft,
                    attributeKinds,
                    panelOpen,
                    () => {
                        void (async () => {
                            const title = draft.title.trim();
                            if (!title) {
                                createError = 'Title is required.';
                                render();
                                return;
                            }
                            try {
                                const created = await createItem({
                                    attributes: draft.attributes,
                                    contextItemId: args.createRow!.contextItemId,
                                    relationship: args.createRow!.relationship,
                                    title,
                                    types: draft.types,
                                });
                                items.unshift(created);
                                const keptTypes = draft.types.slice();
                                draft = { attributes: {}, title: '', types: keptTypes };
                                panelOpen = true;
                                createError = null;
                            } catch (error) {
                                createError = (error as Error).message ?? 'Failed to create item.';
                            }
                            render();
                        })();
                    },
                    () => {
                        panelOpen = false;
                        draft = { attributes: {}, title: '', types: [] };
                    },
                ));
            }
        };

        render();

        // Load attribute kinds for the add panel's type-required fields, then re-render
        if (args.createRow) {
            void loadAttributeKinds().then((kinds) => {
                attributeKinds = kinds;
                render();
            });
        }
    }

    private getParentItemId(item: Item): number | null {
        const parentLink = item.Links.find((link) => link.Relationship === 'parent');
        return parentLink?.OtherItemID ?? null;
    }
}

customElements.define('item-screen', ItemScreen);

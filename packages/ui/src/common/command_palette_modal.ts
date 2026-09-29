import { commands } from '@websoil/engine';
import type { UICommand } from '@websoil/engine/src/ui/commands';
import { icons } from '@zealot/content';

function iconForCommand(name: string): string {
    const n = name.toLowerCase();
    if (n.includes('home')) return icons.home;
    if (n.includes('planner') || n.includes('today') || n.includes('week') || n.includes('month') || n.includes('year') || n.includes('calendar')) return icons.calendar;
    if (n.includes('analysis') || n.includes('statistics')) return icons.statistics;
    if (n.includes('rules')) return icons.rules;
    if (n.includes('settings')) return icons.settings;
    if (n.includes('media') || n.includes('folder')) return icons.folder;
    if (n.includes('types') || n.includes('type') || n.includes('tag')) return icons.tag;
    if (n.startsWith('item:')) return icons.items;
    if (n.includes('search')) return icons.search;
    if (n.includes('new item') || n.includes('add')) return icons.add;
    if (n.includes('random')) return icons.bolt;
    if (n.includes('nav') || n.includes('navigation')) return icons.navigation;
    if (n.includes('note')) return icons.notes;
    if (n.includes('recent')) return icons.recent;
    return icons.bolt;
}

export class CommandPaletteModal extends HTMLElement {
    private rendered = false;
    private input: HTMLInputElement | null = null;
    private listEl: HTMLElement | null = null;
    private selectedIndex = 0;
    private currentCommands: UICommand[] = [];

    static show(): CommandPaletteModal {
        const existing = document.querySelector('command-palette-modal') as CommandPaletteModal | null;
        if (existing) {
            existing.focusInput();
            return existing;
        }
        const modal = new CommandPaletteModal();
        (document.body ?? document.documentElement).appendChild(modal);
        queueMicrotask(() => modal.focusInput());
        return modal;
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
        }
    }

    private render(): void {
        this.rendered = true;
        this.classList.add('modal_background');
        this.classList.add('command-palette-modal');
        this.innerHTML = `
        <div class="inner_window command-palette-window" role="dialog" aria-modal="true" aria-label="Command Runner">
            <div class="command-palette-search-row">
                <img src="${icons.bolt}" alt="" class="command-palette-icon" aria-hidden="true" />
                <input
                    type="text"
                    class="command-palette-input"
                    placeholder="Type a command…"
                    autocomplete="off"
                    spellcheck="false"
                />
            </div>
            <div class="command-palette-list" role="listbox" aria-label="Commands"></div>
        </div>
        `;

        this.input = this.querySelector<HTMLInputElement>('.command-palette-input');
        this.listEl = this.querySelector<HTMLElement>('.command-palette-list');

        this.input?.addEventListener('input', () => this.updateList());
        this.input?.addEventListener('keydown', (e) => this.onInputKeydown(e));

        this.addEventListener('click', (e) => {
            if (e.target === this) this.close();
        });

        this.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                this.close();
            }
        });

        this.updateList();
    }

    private updateList(): void {
        const term = this.input?.value ?? '';
        this.currentCommands = commands.runner.search_commands(term);
        this.selectedIndex = 0;
        this.renderList();
    }

    private renderList(): void {
        if (!this.listEl) return;
        this.listEl.innerHTML = '';

        if (this.currentCommands.length === 0) {
            const empty = document.createElement('div');
            empty.className = 'command-palette-empty';
            empty.textContent = 'No commands found';
            this.listEl.appendChild(empty);
            return;
        }

        this.currentCommands.forEach((cmd, i) => {
            const row = document.createElement('div');
            row.className = 'command-palette-row' + (i === this.selectedIndex ? ' command-palette-row--selected' : '');
            row.setAttribute('role', 'option');
            row.setAttribute('aria-selected', String(i === this.selectedIndex));

            const icon = document.createElement('img');
            icon.src = iconForCommand(cmd.name);
            icon.alt = '';
            icon.className = 'command-palette-row-icon';
            icon.setAttribute('aria-hidden', 'true');

            const label = document.createElement('span');
            label.className = 'command-palette-row-label';
            label.textContent = cmd.name;

            row.appendChild(icon);
            row.appendChild(label);

            const firstHotkey = cmd.hotkeys[0];
            if (firstHotkey) {
                const badge = document.createElement('kbd');
                badge.className = 'command-palette-hotkey';
                badge.textContent = firstHotkey.toString();
                row.appendChild(badge);
            }

            row.addEventListener('click', () => {
                this.close();
                commands.runner.run(cmd.name);
            });

            row.addEventListener('mouseenter', () => {
                this.selectedIndex = i;
                this.highlightSelected();
            });

            this.listEl!.appendChild(row);
        });
    }

    private highlightSelected(): void {
        const rows = this.listEl?.querySelectorAll<HTMLElement>('.command-palette-row');
        rows?.forEach((row, i) => {
            row.classList.toggle('command-palette-row--selected', i === this.selectedIndex);
            row.setAttribute('aria-selected', String(i === this.selectedIndex));
        });
        rows?.[this.selectedIndex]?.scrollIntoView({ block: 'nearest' });
    }

    private onInputKeydown(e: KeyboardEvent): void {
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            this.selectedIndex = Math.min(this.selectedIndex + 1, this.currentCommands.length - 1);
            this.highlightSelected();
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
            this.highlightSelected();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            const cmd = this.currentCommands[this.selectedIndex];
            if (cmd) {
                this.close();
                commands.runner.run(cmd.name);
            }
        }
    }

    private focusInput(): void {
        window.requestAnimationFrame(() => {
            this.input?.focus();
        });
    }

    private close(): void {
        this.remove();
    }
}

if (!customElements.get('command-palette-modal')) {
    customElements.define('command-palette-modal', CommandPaletteModal);
}

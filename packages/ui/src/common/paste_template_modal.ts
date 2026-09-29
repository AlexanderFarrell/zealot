import { Popups } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';

const itemApi = new ItemAPI('/api');

export class PasteTemplateModal extends HTMLElement {
    private onSelect: ((content: string) => void) | null = null;
    private listEl: HTMLElement | null = null;
    private rendered = false;

    static show(onSelect: (content: string) => void): PasteTemplateModal {
        const existing = document.querySelector('paste-template-modal') as PasteTemplateModal | null;
        if (existing) {
            existing.remove();
        }
        const modal = new PasteTemplateModal();
        modal.onSelect = onSelect;
        (document.body ?? document.documentElement).appendChild(modal);
        return modal;
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
            void this.loadTemplates();
        }
    }

    private render(): void {
        this.rendered = true;
        this.classList.add('modal_background');
        this.innerHTML = `
        <div class="inner_window paste-template-modal-window" role="dialog" aria-modal="true" aria-label="Paste Template">
            <div class="tool-panel">
                <div class="tool-panel-header">
                    <h2>Paste Template</h2>
                    <button type="button" class="paste-template-modal-close" aria-label="Close">&times;</button>
                </div>
                <div class="paste-template-list">Loading…</div>
            </div>
        </div>
        `;

        this.listEl = this.querySelector('.paste-template-list');

        this.addEventListener('click', (event) => {
            if (event.target === this) this.close();
        });
        this.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                this.close();
            }
        });
        this.querySelector('.paste-template-modal-close')?.addEventListener('click', () => this.close());
    }

    private async loadTemplates(): Promise<void> {
        if (!this.listEl) return;

        try {
            const templates = await itemApi.GetAll('Template');
            this.listEl.innerHTML = '';

            if (templates.length === 0) {
                const msg = document.createElement('p');
                msg.className = 'tool-muted';
                msg.textContent = 'No templates found. Create an item and assign it the "Template" type.';
                this.listEl.appendChild(msg);
                return;
            }

            const sorted = [...templates].sort((a, b) => a.Title.localeCompare(b.Title));

            for (const template of sorted) {
                const row = document.createElement('div');
                row.className = 'assign-type-row';

                const nameEl = document.createElement('span');
                nameEl.className = 'assign-type-name';
                nameEl.textContent = template.Title;

                const btn = document.createElement('button');
                btn.type = 'button';
                btn.textContent = 'Use';
                btn.className = 'assign-type-btn assign-type-btn--add';
                btn.addEventListener('click', () => {
                    this.onSelect?.(template.Content);
                    this.close();
                });

                row.appendChild(nameEl);
                row.appendChild(btn);
                this.listEl.appendChild(row);
            }
        } catch (e) {
            if (this.listEl) {
                this.listEl.innerHTML = '';
                const err = document.createElement('p');
                err.className = 'tool-error';
                err.textContent = 'Failed to load templates.';
                this.listEl.appendChild(err);
            }
            Popups.add_error((e as Error).message ?? 'Failed to load templates.');
        }
    }

    private close(): void {
        this.remove();
    }
}

if (!customElements.get('paste-template-modal')) {
    customElements.define('paste-template-modal', PasteTemplateModal);
}

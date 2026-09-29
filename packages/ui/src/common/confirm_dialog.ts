export class ConfirmDialog extends HTMLElement {
    private _resolve: ((value: boolean) => void) | null = null;
    private readonly _handleKeydown = (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
            this._close(false);
        }
    };

    static show(message: string): Promise<boolean> {
        return new Promise((resolve) => {
            const dialog = new ConfirmDialog();
            dialog._resolve = resolve;
            dialog.setAttribute('data-message', message);
            (document.body ?? document.documentElement).appendChild(dialog);
        });
    }

    connectedCallback() {
        if (this.shadowRoot) {
            return;
        }

        const message = this.getAttribute('data-message') ?? 'Are you sure?';
        const shadow = this.attachShadow({ mode: 'open' });
        shadow.innerHTML = `
            <style>
                :host {
                    position: fixed;
                    inset: 0;
                    z-index: 1000;
                }
                .overlay {
                    position: fixed;
                    inset: 0;
                    background: rgba(0, 0, 0, 0.5);
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    z-index: 1000;
                }
                .card {
                    background: var(--bg-3, #fff);
                    border: 1px solid var(--bg-4, #d8d8d8);
                    border-radius: 10px;
                    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
                    padding: 1.5rem;
                    max-width: 400px;
                    width: 90%;
                    display: flex;
                    flex-direction: column;
                    gap: 1rem;
                }
                p {
                    margin: 0;
                    color: var(--fg-0, #000);
                }
                .buttons {
                    display: flex;
                    justify-content: flex-end;
                    gap: 0.5rem;
                }
                button {
                    padding: 0.4rem 1rem;
                    border-radius: 4px;
                    border: none;
                    cursor: pointer;
                    font-size: 0.9rem;
                }
                .cancel {
                    background: var(--bg-3, #ddd);
                    color: var(--fg-0, #000);
                }
                .confirm {
                    background: var(--error-0, #d00);
                    color: #fff;
                }
            </style>
            <div class="overlay">
                <div class="card" role="dialog" aria-modal="true" aria-label="Confirmation dialog">
                    <p data-role="message"></p>
                    <div class="buttons">
                        <button class="cancel">Cancel</button>
                        <button class="confirm">Confirm</button>
                    </div>
                </div>
            </div>
        `;

        shadow.querySelector('[data-role="message"]')!.textContent = message;
        shadow.querySelector('.cancel')!.addEventListener('click', () => this._close(false));
        shadow.querySelector('.confirm')!.addEventListener('click', () => this._close(true));

        const overlay = shadow.querySelector('.overlay')!;
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) {
                this._close(false);
            }
        });

        document.addEventListener('keydown', this._handleKeydown);
        (shadow.querySelector('.confirm') as HTMLButtonElement).focus();
    }

    disconnectedCallback(): void {
        document.removeEventListener('keydown', this._handleKeydown);
    }

    private _close(result: boolean): void {
        const resolve = this._resolve;
        this._resolve = null;
        this.remove();
        resolve?.(result);
    }
}

if (!customElements.get('confirm-dialog')) {
    customElements.define('confirm-dialog', ConfirmDialog);
}

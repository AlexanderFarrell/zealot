export class LoadingSpinner extends HTMLElement {
    connectedCallback() {
        if (this.shadowRoot) {
            return;
        }

        const shadow = this.attachShadow({ mode: 'open' });
        shadow.innerHTML = `
            <style>
                :host {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    padding: 2rem;
                }
                .spinner {
                    width: 32px;
                    height: 32px;
                    border: 3px solid var(--bg-3, #ccc);
                    border-top-color: var(--fg-0, #333);
                    border-radius: 50%;
                    animation: spin 0.7s linear infinite;
                }
                @keyframes spin {
                    to { transform: rotate(360deg); }
                }
            </style>
            <div class="spinner"></div>
        `;
    }
}

if (!customElements.get('loading-spinner')) {
    customElements.define('loading-spinner', LoadingSpinner);
}

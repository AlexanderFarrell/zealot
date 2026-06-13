import { getDesktopMode, getRemoteHost } from '@websoil/engine';

export class DesktopModeIndicator extends HTMLElement {
    connectedCallback() {
        if (this.shadowRoot) return;

        const mode = getDesktopMode();
        if (mode === null) {
            // Plain web app — hide entirely
            this.style.display = 'none';
            return;
        }

        const label = mode === 'remote'
            ? `Remote${getRemoteHost() ? ` — ${getRemoteHost()}` : ''}`
            : 'Local';

        const shadow = this.attachShadow({ mode: 'open' });
        shadow.innerHTML = `
            <style>
                :host {
                    display: inline-flex;
                    align-items: center;
                }
                .pill {
                    display: inline-flex;
                    align-items: center;
                    padding: 2px 8px;
                    border-radius: 10px;
                    font-size: 11px;
                    font-weight: 600;
                    letter-spacing: 0.03em;
                    user-select: none;
                    background: var(--mode-pill-bg, #e0e7ff);
                    color: var(--mode-pill-fg, #3730a3);
                }
                :host([data-mode="local"]) .pill {
                    background: var(--mode-pill-local-bg, #dcfce7);
                    color: var(--mode-pill-local-fg, #166534);
                }
            </style>
            <span class="pill">${label}</span>
        `;
        this.setAttribute('data-mode', mode);
    }
}

if (!customElements.get('zealot-mode-indicator')) {
    customElements.define('zealot-mode-indicator', DesktopModeIndicator);
}

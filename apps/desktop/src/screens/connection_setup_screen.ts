import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { BaseElement } from '@websoil/engine';
import type { DesktopMode } from '@websoil/engine';
import { initWithServerUrl } from '../desktop_core';

interface ConnectionSetupData {
    onComplete: () => void;
}

export class ConnectionSetupScreen extends BaseElement<ConnectionSetupData> {
    private selectedMode: DesktopMode = 'remote';

    render() {
        this.className = 'modal_background auth_modal';
        this.innerHTML = `
        <form id="connection_setup_form" class="inner_window">
            <h1 style="text-align: center;">Connect to Zealot</h1>
            <p style="text-align: center; color: var(--text-muted);">
                Choose how to connect to your Zealot server.
            </p>

            <div id="mode_selector" style="display:flex; gap: 8px; margin-bottom: 16px;">
                <button type="button" id="btn_remote" class="mode-btn active" style="flex:1;">
                    Fully Remote
                </button>
                <button type="button" id="btn_local" class="mode-btn" style="flex:1;">
                    Local Server
                </button>
            </div>

            <p id="mode_description" style="color: var(--text-muted); font-size: 0.9em; margin-bottom: 12px;">
                Connect to an external Zealot server. Only your server URL and API key are stored locally.
            </p>

            <label for="server_url">Server URL</label>
            <input
                type="url"
                id="server_url"
                name="server_url"
                placeholder="https://zealot.example.com"
                autocomplete="url"
                autocorrect="off"
                autocapitalize="none"
                spellcheck="false"
            >
            <button type="submit">Connect</button>
            <p id="setup_error" class="error" style="display:none;"></p>
        </form>
        `;

        const form      = this.querySelector<HTMLFormElement>('#connection_setup_form')!;
        const errorEl   = this.querySelector<HTMLParagraphElement>('#setup_error')!;
        const submitBtn = form.querySelector<HTMLButtonElement>('button[type="submit"]')!;
        const btnRemote = this.querySelector<HTMLButtonElement>('#btn_remote')!;
        const btnLocal  = this.querySelector<HTMLButtonElement>('#btn_local')!;
        const modeDesc  = this.querySelector<HTMLParagraphElement>('#mode_description')!;
        const urlInput  = this.querySelector<HTMLInputElement>('#server_url')!;

        const remoteDesc = 'Connect to an external Zealot server. Only your server URL and API key are stored locally.';
        const localDesc  = 'Connect to a local Zealot server running on this machine. Structured for future local server management.';

        btnRemote.addEventListener('click', () => {
            this.selectedMode = 'remote';
            btnRemote.classList.add('active');
            btnLocal.classList.remove('active');
            modeDesc.textContent = remoteDesc;
            urlInput.placeholder = 'https://zealot.example.com';
        });

        btnLocal.addEventListener('click', () => {
            this.selectedMode = 'local';
            btnLocal.classList.add('active');
            btnRemote.classList.remove('active');
            modeDesc.textContent = localDesc;
            urlInput.placeholder = 'http://localhost:8456';
        });

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            void this.handleSubmit(form, errorEl, submitBtn);
        });
    }

    private async handleSubmit(
        form: HTMLFormElement,
        errorEl: HTMLParagraphElement,
        submitBtn: HTMLButtonElement,
    ): Promise<void> {
        const data   = new FormData(form);
        const rawUrl = (data.get('server_url') as string).trim().replace(/\/$/, '');

        errorEl.style.display = 'none';
        errorEl.textContent   = '';
        submitBtn.disabled    = true;
        submitBtn.textContent = 'Connecting…';

        try {
            const resp = await tauriFetch(`${rawUrl}/auth/is_logged_in`, { method: 'GET' });
            // 200 (logged in) or 401 (not logged in) both confirm the server is reachable
            if (!resp.ok && resp.status !== 401) {
                throw new Error(`Server responded with status ${resp.status}`);
            }
            await initWithServerUrl(rawUrl, this.selectedMode);
            this.data!.onComplete();
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Could not reach server.';
            errorEl.textContent   = `Connection failed: ${msg}`;
            errorEl.style.display = '';
            submitBtn.disabled    = false;
            submitBtn.textContent = 'Connect';
        }
    }
}

customElements.define('connection-setup-screen', ConnectionSetupScreen);

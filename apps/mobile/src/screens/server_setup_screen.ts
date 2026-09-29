import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { BaseElement } from '@websoil/engine';
import { initWithServerUrl } from '../mobile_core';

interface ServerSetupData {
    onComplete: () => void;
}

export class ServerSetupScreen extends BaseElement<ServerSetupData> {
    render() {
        this.className = 'modal_background auth_modal';
        this.innerHTML = `
        <form id="server_setup_form" class="inner_window">
            <h1 style="text-align: center;">Connect to Zealot</h1>
            <p style="text-align: center; color: var(--text-muted);">
                Enter the URL of your self-hosted Zealot server.
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

        const form = this.querySelector<HTMLFormElement>('#server_setup_form')!;
        const errorEl = this.querySelector<HTMLParagraphElement>('#setup_error')!;
        const submitBtn = form.querySelector<HTMLButtonElement>('button[type="submit"]')!;

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
        const data = new FormData(form);
        const rawUrl = (data.get('server_url') as string).trim().replace(/\/$/, '');

        errorEl.style.display = 'none';
        errorEl.textContent = '';
        submitBtn.disabled = true;
        submitBtn.textContent = 'Connecting…';

        try {
            const resp = await tauriFetch(`${rawUrl}/auth/is_logged_in`, { method: 'GET' });
            // 200 (logged in) or 401 (not logged in) both confirm the server is reachable
            if (!resp.ok && resp.status !== 401) {
                throw new Error(`Server responded with status ${resp.status}`);
            }
            await initWithServerUrl(rawUrl);
            this.data!.onComplete();
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Could not reach server.';
            errorEl.textContent = `Connection failed: ${msg}`;
            errorEl.style.display = '';
            submitBtn.disabled = false;
            submitBtn.textContent = 'Connect';
        }
    }
}

customElements.define('server-setup-screen', ServerSetupScreen);

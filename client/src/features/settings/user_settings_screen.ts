import AuthAPI from "../../api/auth";

class UserSettingsScreen extends HTMLElement {
    private keyDisplay: HTMLElement | null = null;

    connectedCallback() {
        this.render();
        this.loadStatus();
    }

    disconnectedCallback() {}

    private render() {
        this.innerHTML = `
            <h1>User Settings</h1>
            <section class="api-key-section">
                <h2>API Key</h2>
                <p class="api-key-description">Use an API key to authenticate requests via the <code>Authorization: Bearer &lt;key&gt;</code> header. One key per account — generating a new key immediately invalidates the old one.</p>
                <div class="api-key-status" id="api-key-status">Checking...</div>
                <div class="api-key-display" id="api-key-display" style="display:none">
                    <p>Your new API key (shown once — copy it now):</p>
                    <div class="api-key-value-row">
                        <code id="api-key-value" class="api-key-value"></code>
                        <button id="api-key-copy" class="btn-secondary">Copy</button>
                    </div>
                </div>
                <div class="api-key-actions">
                    <button id="api-key-generate" class="btn-primary">Generate New Key</button>
                    <button id="api-key-revoke" class="btn-danger" style="display:none">Revoke Key</button>
                </div>
            </section>
        `;
        this.keyDisplay = this.querySelector('#api-key-display');
        this.querySelector('#api-key-generate')?.addEventListener('click', () => this.generate());
        this.querySelector('#api-key-revoke')?.addEventListener('click', () => this.revoke());
        this.querySelector('#api-key-copy')?.addEventListener('click', () => this.copyKey());
    }

    private async loadStatus() {
        try {
            const { exists } = await AuthAPI.get_api_key_status();
            this.setStatus(exists);
        } catch {
            this.setStatusText('Failed to load API key status.');
        }
    }

    private setStatus(exists: boolean) {
        this.setStatusText(exists ? 'An API key exists for your account.' : 'No API key set.');
        const revokeBtn = this.querySelector<HTMLElement>('#api-key-revoke');
        if (revokeBtn) revokeBtn.style.display = exists ? '' : 'none';
        if (this.keyDisplay) this.keyDisplay.style.display = 'none';
    }

    private setStatusText(text: string) {
        const el = this.querySelector('#api-key-status');
        if (el) el.textContent = text;
    }

    private async generate() {
        if (!confirm('Generate a new API key? Any existing key will be immediately invalidated.')) return;
        try {
            const { key } = await AuthAPI.generate_api_key();
            const keyEl = this.querySelector<HTMLElement>('#api-key-value');
            if (keyEl) keyEl.textContent = key;
            if (this.keyDisplay) this.keyDisplay.style.display = '';
            this.setStatus(true);
            this.setStatusText('New API key generated. Copy it now — it will not be shown again.');
        } catch {
            // error already surfaced via popup
        }
    }

    private async revoke() {
        if (!confirm('Revoke your API key? This cannot be undone.')) return;
        try {
            await AuthAPI.revoke_api_key();
            this.setStatus(false);
        } catch {
            // error already surfaced via popup
        }
    }

    private copyKey() {
        const key = this.querySelector<HTMLElement>('#api-key-value')?.textContent ?? '';
        navigator.clipboard.writeText(key).then(() => {
            const btn = this.querySelector<HTMLButtonElement>('#api-key-copy');
            if (btn) {
                btn.textContent = 'Copied!';
                setTimeout(() => { btn.textContent = 'Copy'; }, 2000);
            }
        });
    }
}

customElements.define('user-settings-screen', UserSettingsScreen);

export default UserSettingsScreen;
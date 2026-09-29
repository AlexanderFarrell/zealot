import { BaseElementEmpty, Events } from '@websoil/engine';
import { AuthAPI } from '@zealot/api/src/auth';
import { ItemAPI } from '@zealot/api/src/item';
import { ApiKey } from '@zealot/domain/src/account';
import { LoadingSpinner } from '../common/loading_spinner';
import { ConfirmDialog } from '../common/confirm_dialog';

let authApi = new AuthAPI('/api');
let itemApi = new ItemAPI('/api');

export function configureUserSettingsAPIs(auth: AuthAPI, item: ItemAPI): void {
    authApi = auth;
    itemApi = item;
}

function isTauriEnv(): boolean {
    return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}

export class UserSettingsScreen extends BaseElementEmpty {
    private renderId = 0;
    private logoutError: string | null = null;
    private loggingOut = false;

    private apiKeys: ApiKey[] = [];
    private apiKeysLoaded = false;
    private newKeyLabel = '';
    private generatedKey: string | null = null;
    private apiKeyBusy = false;
    private apiKeyError: string | null = null;

    private rebuildBusy = false;
    private rebuildResult: string | null = null;
    private rebuildError: string | null = null;

    async render() {
        const renderId = ++this.renderId;

        this.className = 'user-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'user-settings-shell';

        const heading = document.createElement('h2');
        heading.textContent = 'User Settings';
        shell.appendChild(heading);

        const spinner = new LoadingSpinner();
        shell.appendChild(spinner);
        this.appendChild(shell);

        let loggedIn: boolean;
        try {
            loggedIn = await authApi.IsLoggedIn();
        } catch {
            loggedIn = false;
        }

        if (renderId !== this.renderId) {
            return;
        }

        shell.removeChild(spinner);

        if (!loggedIn || !authApi.Account) {
            Events.emit('to_auth');
            return;
        }

        const account = authApi.Account;

        if (!this.apiKeysLoaded) {
            try {
                this.apiKeys = await authApi.listApiKeys();
            } catch {
                // non-fatal — show empty list
            }
            this.apiKeysLoaded = true;
        }

        if (renderId !== this.renderId) {
            return;
        }

        const infoTable = document.createElement('table');
        infoTable.className = 'user-info-table';
        infoTable.innerHTML = `
            <tbody>
                <tr><th>Username</th><td>${escapeHtml(account.Username)}</td></tr>
                <tr><th>Email</th><td>${escapeHtml(account.Email)}</td></tr>
                <tr><th>Given Name</th><td>${escapeHtml(account.GivenName)}</td></tr>
                <tr><th>Surname</th><td>${escapeHtml(account.Surname)}</td></tr>
            </tbody>
        `;
        shell.appendChild(infoTable);

        if (!isTauriEnv()) {
            shell.appendChild(this.renderApiKeySection());
        }
        shell.appendChild(this.renderAdminSection());

        if (this.logoutError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = this.logoutError;
            shell.appendChild(error);
        }

        const logoutButton = document.createElement('button');
        logoutButton.type = 'button';
        logoutButton.textContent = this.loggingOut ? 'Logging out…' : 'Log Out';
        logoutButton.disabled = this.loggingOut;
        logoutButton.addEventListener('click', () => {
            void this.logout();
        });
        shell.appendChild(logoutButton);
    }

    private renderApiKeySection(): HTMLElement {
        const section = document.createElement('div');
        section.className = 'api-key-section';

        const sectionHeading = document.createElement('h3');
        sectionHeading.textContent = 'API Keys';
        section.appendChild(sectionHeading);

        if (this.generatedKey !== null) {
            const banner = document.createElement('div');
            banner.className = 'api-key-banner';

            const warning = document.createElement('p');
            warning.className = 'tool-muted';
            warning.textContent = 'Save this key now — it will not be shown again.';
            banner.appendChild(warning);

            const keyRow = document.createElement('div');
            keyRow.className = 'api-key-row';

            const keyInput = document.createElement('input');
            keyInput.type = 'text';
            keyInput.readOnly = true;
            keyInput.value = this.generatedKey;
            keyInput.className = 'api-key-input';
            keyRow.appendChild(keyInput);

            const copyButton = document.createElement('button');
            copyButton.type = 'button';
            copyButton.textContent = 'Copy';
            copyButton.addEventListener('click', () => {
                void navigator.clipboard.writeText(this.generatedKey ?? '');
                copyButton.textContent = 'Copied!';
                setTimeout(() => { copyButton.textContent = 'Copy'; }, 2000);
            });
            keyRow.appendChild(copyButton);
            banner.appendChild(keyRow);
            section.appendChild(banner);
        }

        if (this.apiKeyError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = this.apiKeyError;
            section.appendChild(error);
        }

        if (this.apiKeys.length > 0) {
            const table = document.createElement('table');
            table.className = 'api-keys-table';

            const thead = document.createElement('thead');
            thead.innerHTML = '<tr><th>Label</th><th>Created</th><th></th></tr>';
            table.appendChild(thead);

            const tbody = document.createElement('tbody');
            for (const key of this.apiKeys) {
                const row = document.createElement('tr');

                const labelCell = document.createElement('td');
                labelCell.textContent = key.Label;
                row.appendChild(labelCell);

                const dateCell = document.createElement('td');
                dateCell.className = 'tool-muted';
                dateCell.textContent = formatDate(key.CreatedAt);
                row.appendChild(dateCell);

                const actionCell = document.createElement('td');
                const revokeBtn = document.createElement('button');
                revokeBtn.type = 'button';
                revokeBtn.className = 'button-danger button-small';
                revokeBtn.textContent = 'Revoke';
                revokeBtn.disabled = this.apiKeyBusy;
                revokeBtn.addEventListener('click', () => { void this.revokeKey(key); });
                actionCell.appendChild(revokeBtn);
                row.appendChild(actionCell);

                tbody.appendChild(row);
            }
            table.appendChild(tbody);
            section.appendChild(table);
        } else {
            const status = document.createElement('p');
            status.className = 'tool-muted';
            status.textContent = 'No API keys configured.';
            section.appendChild(status);
        }

        const form = document.createElement('div');
        form.className = 'api-key-form';

        const labelInput = document.createElement('input');
        labelInput.type = 'text';
        labelInput.placeholder = 'Label (e.g. MCP Server)';
        labelInput.value = this.newKeyLabel;
        labelInput.className = 'api-key-label-input';
        labelInput.addEventListener('input', () => { this.newKeyLabel = labelInput.value; });
        form.appendChild(labelInput);

        const generateButton = document.createElement('button');
        generateButton.type = 'button';
        generateButton.textContent = this.apiKeyBusy ? 'Generating…' : 'Generate Key';
        generateButton.disabled = this.apiKeyBusy;
        generateButton.addEventListener('click', () => { void this.generateKey(labelInput.value); });
        form.appendChild(generateButton);

        section.appendChild(form);

        return section;
    }

    private renderAdminSection(): HTMLElement {
        const section = document.createElement('div');
        section.className = 'admin-section';

        const heading = document.createElement('h3');
        heading.textContent = 'Maintenance';
        section.appendChild(heading);

        const desc = document.createElement('p');
        desc.className = 'tool-muted';
        desc.textContent = 'Rebuild the item relationship index from attribute values and wiki links. Use this if children, linked items, or backlinks are missing.';
        section.appendChild(desc);

        if (this.rebuildResult) {
            const ok = document.createElement('p');
            ok.className = 'tool-muted';
            ok.textContent = this.rebuildResult;
            section.appendChild(ok);
        }
        if (this.rebuildError) {
            const err = document.createElement('p');
            err.className = 'tool-error';
            err.textContent = this.rebuildError;
            section.appendChild(err);
        }

        const button = document.createElement('button');
        button.type = 'button';
        button.textContent = this.rebuildBusy ? 'Rebuilding…' : 'Rebuild Links';
        button.disabled = this.rebuildBusy;
        button.addEventListener('click', () => { void this.rebuildLinks(); });
        section.appendChild(button);

        return section;
    }

    private async rebuildLinks(): Promise<void> {
        if (this.rebuildBusy) return;
        this.rebuildBusy = true;
        this.rebuildResult = null;
        this.rebuildError = null;
        void this.render();

        try {
            const result = await itemApi.RebuildLinks();
            this.rebuildResult = `Done — rebuilt links for ${result.rebuilt} item(s).`;
        } catch (error) {
            this.rebuildError = error instanceof Error && error.message
                ? error.message
                : 'Rebuild failed.';
        }

        this.rebuildBusy = false;
        void this.render();
    }

    private async generateKey(labelValue: string): Promise<void> {
        if (this.apiKeyBusy) return;
        this.apiKeyBusy = true;
        this.apiKeyError = null;
        void this.render();

        try {
            const label = labelValue.trim() || 'Default';
            const result = await authApi.createApiKey(label);
            this.generatedKey = result.key;
            this.newKeyLabel = '';
            this.apiKeys = await authApi.listApiKeys();
        } catch (error) {
            this.apiKeyError = error instanceof Error && error.message
                ? error.message
                : 'Failed to generate API key.';
        }

        this.apiKeyBusy = false;
        void this.render();
    }

    private async revokeKey(key: ApiKey): Promise<void> {
        if (this.apiKeyBusy) return;
        const confirmed = await ConfirmDialog.show(`Revoke key "${key.Label}"? Integrations using it will stop working.`);
        if (!confirmed) return;

        this.apiKeyBusy = true;
        this.apiKeyError = null;
        if (this.generatedKey !== null) {
            this.generatedKey = null;
        }
        void this.render();

        try {
            await authApi.deleteApiKey(key.ApiKeyId);
            this.apiKeys = this.apiKeys.filter(k => k.ApiKeyId !== key.ApiKeyId);
        } catch (error) {
            this.apiKeyError = error instanceof Error && error.message
                ? error.message
                : 'Failed to revoke API key.';
        }

        this.apiKeyBusy = false;
        void this.render();
    }

    private async logout(): Promise<void> {
        if (this.loggingOut) {
            return;
        }
        this.loggingOut = true;
        this.logoutError = null;
        void this.render();

        try {
            await authApi.Basic.logout();
            Events.emit('to_auth');
        } catch (error) {
            this.logoutError = error instanceof Error && error.message
                ? error.message
                : 'Logout failed.';
            this.loggingOut = false;
            void this.render();
        }
    }
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

function formatDate(iso: string): string {
    try {
        return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    } catch {
        return iso;
    }
}

if (!customElements.get('user-settings-screen')) {
    customElements.define('user-settings-screen', UserSettingsScreen);
}

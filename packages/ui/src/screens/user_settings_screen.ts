import { BaseElementEmpty, Events } from '@websoil/engine';
import { AuthAPI } from '@zealot/api/src/auth';
import { ItemAPI } from '@zealot/api/src/item';
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

    private hasApiKey: boolean | null = null;
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

        if (this.hasApiKey === null) {
            this.hasApiKey = account.HasApiKey;
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
        sectionHeading.textContent = 'API Key';
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

        if (this.hasApiKey) {
            const status = document.createElement('p');
            status.className = 'tool-muted';
            status.textContent = this.generatedKey
                ? 'A new API key has been generated above.'
                : 'An API key is currently active.';
            section.appendChild(status);

            const revokeButton = document.createElement('button');
            revokeButton.type = 'button';
            revokeButton.className = 'button-danger';
            revokeButton.textContent = this.apiKeyBusy ? 'Revoking…' : 'Revoke Key';
            revokeButton.disabled = this.apiKeyBusy;
            revokeButton.addEventListener('click', () => { void this.revokeKey(); });
            section.appendChild(revokeButton);
        } else {
            const status = document.createElement('p');
            status.className = 'tool-muted';
            status.textContent = 'No API key configured.';
            section.appendChild(status);

            const generateButton = document.createElement('button');
            generateButton.type = 'button';
            generateButton.textContent = this.apiKeyBusy ? 'Generating…' : 'Generate Key';
            generateButton.disabled = this.apiKeyBusy;
            generateButton.addEventListener('click', () => { void this.generateKey(); });
            section.appendChild(generateButton);
        }

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
        desc.textContent = 'Rebuild the item relationship index from attribute values. Use this if children or linked items are missing from navigation.';
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

    private async generateKey(): Promise<void> {
        if (this.apiKeyBusy) return;
        this.apiKeyBusy = true;
        this.apiKeyError = null;
        void this.render();

        try {
            this.generatedKey = await authApi.createApiKey();
            this.hasApiKey = true;
        } catch (error) {
            this.apiKeyError = error instanceof Error && error.message
                ? error.message
                : 'Failed to generate API key.';
        }

        this.apiKeyBusy = false;
        void this.render();
    }

    private async revokeKey(): Promise<void> {
        if (this.apiKeyBusy) return;
        const confirmed = await ConfirmDialog.show('Revoke your API key? Any integrations using it will stop working.');
        if (!confirmed) return;

        this.apiKeyBusy = true;
        this.apiKeyError = null;
        void this.render();

        try {
            await authApi.deleteApiKey();
            this.hasApiKey = false;
            this.generatedKey = null;
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

if (!customElements.get('user-settings-screen')) {
    customElements.define('user-settings-screen', UserSettingsScreen);
}

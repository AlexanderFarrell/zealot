import { BaseElementEmpty, Events } from '@websoil/engine';
import { AuthAPI } from '@zealot/api/src/auth';
import { LoadingSpinner } from '../common/loading_spinner';

const authApi = new AuthAPI('/api');

export class UserSettingsScreen extends BaseElementEmpty {
    private renderId = 0;
    private logoutError: string | null = null;
    private loggingOut = false;

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

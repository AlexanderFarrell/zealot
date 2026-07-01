import 'dockview-core/dist/styles/dockview.css';
import { Events, Popups } from '@websoil/engine';
import { DesktopClient } from './desktop_client';
import { DesktopLoginScreen } from './screens/desktop_login_screen';
import { ConnectionSetupScreen } from './screens/connection_setup_screen';
import { tryLoadCredentials, completeLoginWithKey, logout, getAPI } from './desktop_core';
import { installFetchOverride } from './desktop_fetch';

// Mark the document so CSS can distinguish Tauri native app from browser.
if (typeof (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ !== 'undefined') {
    document.body.classList.add('tauri-app');
}

// Route all API calls through Tauri's native HTTP client (reqwest), bypassing WebView CORS.
installFetchOverride();

window.addEventListener('unhandledrejection', (e) => {
    console.error('Unhandled promise rejection:', e.reason);
    Popups.add_error(e.reason?.message ?? 'An unexpected error occurred.');
});

const appEl = document.querySelector<HTMLDivElement>('#app')!;
appEl.innerHTML = '';

async function boot(): Promise<void> {
    appEl.innerHTML = '';

    const hasCredentials = await tryLoadCredentials();

    if (!hasCredentials) {
        showConnectionSetup();
        return;
    }

    // Verify the stored API key is still valid before skipping the login screen
    const api = getAPI();
    let loggedIn = false;
    try {
        loggedIn = await api.Auth.IsLoggedIn();
    } catch {
        loggedIn = false;
    }

    if (!loggedIn) {
        // Stale credentials — clear and start over
        await logout();
        showConnectionSetup();
        return;
    }

    showDirectApp();
}

function showConnectionSetup(): void {
    const setup = new ConnectionSetupScreen();
    setup.init({
        onComplete: () => {
            appEl.innerHTML = '';
            showAuthPhase();
        },
    });
    appEl.appendChild(setup);
}

function showAuthPhase(): void {
    const login = new DesktopLoginScreen();
    login.init({
        onLogin: async (username: string, password: string) => {
            const result = await getAPI().Auth.createApiKeyWithCredentials(username, password, 'Desktop');
            await completeLoginWithKey(result.key, result.api_key_id);
            appEl.innerHTML = '';
            showDirectApp();
        },
    });
    appEl.appendChild(login);
}

function showDirectApp(): void {
    // Register a one-shot logout handler. The handler removes itself so that
    // a fresh boot() cycle gets a clean slate.
    const onToAuth = async (): Promise<void> => {
        Events.off('to_auth', onToAuth);
        await logout();
        void boot();
    };
    Events.on('to_auth', onToAuth);

    appEl.appendChild(new DesktopClient());
}

void boot();

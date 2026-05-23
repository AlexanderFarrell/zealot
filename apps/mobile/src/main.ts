import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { Events, Popups } from '@websoil/engine';
import { MobileClient } from './mobile_client';
import { MobileLoginScreen } from './screens/mobile_login_screen';
import { ServerSetupScreen } from './screens/server_setup_screen';
import { tryLoadCredentials, completeLoginWithKey, logout, getAPI, getServerUrl } from './mobile_core';

// Route all API calls through Tauri's native HTTP client (reqwest), bypassing WKWebView CORS.
//
// UI components use relative URLs like /api/item/title/Home (hardcoded for the web app).
// The browser resolves these to tauri://localhost/api/... — not the real server.
// We resolve all URLs against tauri://localhost first, then rewrite tauri://localhost/api/...
// to the actual Zealot server origin so tauriFetch reaches the right host.
const _browserFetch = fetch;
(window as unknown as { __zealotFetch: typeof fetch }).__zealotFetch = (input, init) => {
    const raw = input instanceof Request ? input.url
        : input instanceof URL ? input.href
        : String(input);

    // Resolve relative URLs so we can inspect the full URL.
    let url: string;
    try {
        url = new URL(raw, 'tauri://localhost').href;
    } catch {
        url = raw;
    }

    // Rewrite tauri://localhost/api/... → serverOrigin/api/...
    if (url.startsWith('tauri://localhost/api/')) {
        const serverUrl = getServerUrl();
        if (serverUrl) {
            const serverOrigin = new URL(serverUrl).origin;
            url = serverOrigin + url.slice('tauri://localhost'.length);
        }
    }

    if (url.startsWith('http://') || url.startsWith('https://')) {
        // Strip browser-only fields — tauriFetch uses new Request(url, init) internally
        // and WebKit throws SyntaxError for credentials/mode on cross-origin tauri:// requests.
        const { credentials, mode, cache, redirect, referrer, referrerPolicy, integrity, keepalive, ...nativeInit } = init ?? {};
        return tauriFetch(url, nativeInit) as Promise<Response>;
    }

    return _browserFetch(input, init);
};

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
        showServerSetup();
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
        showServerSetup();
        return;
    }

    showDirectApp();
}

function showServerSetup(): void {
    const setup = new ServerSetupScreen();
    setup.init({
        onComplete: () => {
            appEl.innerHTML = '';
            showAuthPhase();
        },
    });
    appEl.appendChild(setup);
}

function showAuthPhase(): void {
    const login = new MobileLoginScreen();
    login.init({
        onLogin: async (username: string, password: string) => {
            const result = await getAPI().Auth.createApiKeyWithCredentials(username, password, 'Mobile');
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

    appEl.appendChild(new MobileClient());
}

void boot();

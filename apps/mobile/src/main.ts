import { Events, Popups } from '@websoil/engine';
import { MobileClient } from './mobile_client';
import { MobileLoginScreen } from './screens/mobile_login_screen';
import { ServerSetupScreen } from './screens/server_setup_screen';
import { tryLoadCredentials, completeLoginWithKey, logout, getAPI } from './mobile_core';

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
            const key = await getAPI().Auth.createApiKeyWithCredentials(username, password);
            await completeLoginWithKey(key);
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

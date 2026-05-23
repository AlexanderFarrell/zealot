import { Events, Popups } from '@websoil/engine';
import { MainScreen } from '@zealot/ui/src/screens/main_screen';
import { MobileClient } from './mobile_client';
import { ServerSetupScreen } from './screens/server_setup_screen';
import { tryLoadCredentials, completeLogin, logout, getAPI } from './mobile_core';

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
    const api = getAPI();

    // Intercept to_app: after session login, generate and store the API key,
    // then transition to the direct-app view (bypassing MainScreen.to_app).
    const onToApp = async (): Promise<void> => {
        Events.off('to_app', onToApp);
        try {
            await completeLogin();
        } catch (e) {
            console.error('Failed to generate API key after login:', e);
            // Non-fatal: app still works via session for this session
        }
        appEl.innerHTML = '';
        showDirectApp();
    };

    Events.on('to_app', onToApp);

    appEl.appendChild(
        new MainScreen().init({
            AuthAPI: api.Auth,
            // MainScreen.to_app() appends this, but we clear appEl immediately after
            MainContentBuilder: () => document.createElement('div'),
        }),
    );
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

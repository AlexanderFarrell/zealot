import { SetApiKey, setDesktopMode } from '@websoil/engine';
import type { DesktopMode } from '@websoil/engine';
import ZealotAPI from '@zealot/api';
import { AuthAPI } from '@zealot/api/src/auth';
import { ItemAPI } from '@zealot/api/src/item';
import { configureUserSettingsAPIs } from '@zealot/ui/src/screens/user_settings_screen';

const KEY_SERVER_URL   = 'zealot_serverUrl';
const KEY_API_KEY      = 'zealot_apiKey';
const KEY_API_KEY_ID   = 'zealot_apiKeyId';
const KEY_DESKTOP_MODE = 'zealot_desktopMode';

let _api: ZealotAPI | null = null;
let _serverUrl: string | null = null;
let _apiKeyId: number | null = null;

function applyApiInstance(api: ZealotAPI, serverUrl: string, mode: DesktopMode): void {
    _api = api;
    _serverUrl = serverUrl;
    setDesktopMode(mode, mode === 'remote' ? serverUrl : undefined);
    configureUserSettingsAPIs(new AuthAPI(serverUrl), new ItemAPI(serverUrl));
}

export async function tryLoadCredentials(): Promise<boolean> {
    const serverUrl    = localStorage.getItem(KEY_SERVER_URL);
    const apiKey       = localStorage.getItem(KEY_API_KEY);
    const apiKeyIdStr  = localStorage.getItem(KEY_API_KEY_ID);
    const modeStr      = localStorage.getItem(KEY_DESKTOP_MODE) as DesktopMode | null;
    if (serverUrl && apiKey) {
        const mode: DesktopMode = modeStr === 'local' ? 'local' : 'remote';
        applyApiInstance(new ZealotAPI(serverUrl), serverUrl, mode);
        SetApiKey(apiKey);
        _apiKeyId = apiKeyIdStr ? parseInt(apiKeyIdStr, 10) : null;
        return true;
    }
    return false;
}

export async function initWithServerUrl(url: string, mode: DesktopMode): Promise<void> {
    localStorage.setItem(KEY_SERVER_URL, url);
    localStorage.setItem(KEY_DESKTOP_MODE, mode);
    applyApiInstance(new ZealotAPI(url), url, mode);
}

export async function completeLoginWithKey(key: string, id: number): Promise<void> {
    localStorage.setItem(KEY_API_KEY, key);
    localStorage.setItem(KEY_API_KEY_ID, String(id));
    SetApiKey(key);
    _apiKeyId = id;
}

export async function logout(): Promise<void> {
    if (_api && _apiKeyId != null) {
        try {
            await _api.Auth.deleteApiKey(_apiKeyId);
        } catch {
            // Best-effort: clear locally regardless of server response
        }
    }
    SetApiKey(null);
    localStorage.removeItem(KEY_API_KEY);
    localStorage.removeItem(KEY_API_KEY_ID);
    localStorage.removeItem(KEY_SERVER_URL);
    localStorage.removeItem(KEY_DESKTOP_MODE);
    _api = null;
    _serverUrl = null;
    _apiKeyId = null;
}

export function getAPI(): ZealotAPI {
    if (!_api) throw new Error('API not initialised — credentials not loaded');
    return _api;
}

export function getServerUrl(): string | null {
    return _serverUrl;
}

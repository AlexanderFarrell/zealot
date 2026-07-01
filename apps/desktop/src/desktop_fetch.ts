import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { getServerUrl } from './desktop_core';

export function installFetchOverride(): void {
    const _browserFetch = fetch;
    (window as unknown as { __zealotFetch: typeof fetch }).__zealotFetch = (input, init) => {
        const raw = input instanceof Request ? input.url
            : input instanceof URL ? input.href
            : String(input);

        let url: string;
        try {
            url = new URL(raw, 'tauri://localhost').href;
        } catch {
            url = raw;
        }

        if (url.startsWith('tauri://localhost/api/')) {
            const serverUrl = getServerUrl();
            if (serverUrl) {
                const serverOrigin = new URL(serverUrl).origin;
                url = serverOrigin + url.slice('tauri://localhost'.length);
            }
        }

        if (url.startsWith('http://') || url.startsWith('https://')) {
            const { credentials, mode, cache, redirect, referrer, referrerPolicy, integrity, keepalive, ...nativeInit } = init ?? {};
            return tauriFetch(url, nativeInit) as Promise<Response>;
        }

        return _browserFetch(input, init);
    };
}

import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { getServerUrl } from './desktop_core';

const API_KEY_STORAGE_KEY = 'zealot_apiKey';
const MEDIA_PATH_PREFIX = '/api/media/';

/**
 * Images are loaded by the WebView itself, so they do not pass through the
 * fetch override below.  In the desktop app media is authenticated with the
 * API key, which an HTMLImageElement cannot attach as a request header.
 *
 * Resolve Zealot media through Tauri's native HTTP client and replace its src
 * with a local object URL. This keeps credentials out of URLs and works for
 * both newly pasted images and images already stored in ZealotScript.
 */
function installMediaImageResolver(): void {
    const resolved = new Map<string, Promise<string | null>>();

    // Only proxy the app's relative media URLs. In particular, never attach
    // the API key to an arbitrary external image whose path happens to match.
    const isMediaUrl = (src: string): boolean =>
        src.startsWith(MEDIA_PATH_PREFIX) || src.startsWith(`tauri://localhost${MEDIA_PATH_PREFIX}`);

    const resolve = (src: string): Promise<string | null> => {
        const existing = resolved.get(src);
        if (existing) return existing;

        const request = (async () => {
            const serverUrl = getServerUrl();
            const apiKey = localStorage.getItem(API_KEY_STORAGE_KEY);
            if (!serverUrl || !apiKey) return null;

            const mediaPath = src.startsWith('tauri://localhost')
                ? src.slice('tauri://localhost'.length)
                : src;
            const url = new URL(mediaPath, serverUrl).href;
            const response = await tauriFetch(url, {
                headers: { 'X-Api-Key': apiKey },
            });
            if (!response.ok) return null;
            return URL.createObjectURL(await response.blob());
        })().catch((error: unknown) => {
            console.warn('Failed to load protected media image', error);
            return null;
        });
        resolved.set(src, request);
        return request;
    };

    const replaceSource = (image: HTMLImageElement): void => {
        const src = image.getAttribute('src');
        if (!src || !isMediaUrl(src) || image.dataset.zealotMediaResolved === 'true') return;

        image.dataset.zealotMediaResolved = 'true';
        void resolve(src).then((objectUrl) => {
            if (objectUrl && image.isConnected) image.src = objectUrl;
        });
    };

    const scan = (root: ParentNode): void => {
        if (root instanceof HTMLImageElement) replaceSource(root);
        root.querySelectorAll?.('img').forEach((image) => replaceSource(image));
    };

    scan(document);
    new MutationObserver((records) => {
        for (const record of records) {
            record.addedNodes.forEach((node) => {
                if (node instanceof HTMLElement) scan(node);
            });
        }
    }).observe(document.documentElement, { childList: true, subtree: true });
}

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

    installMediaImageResolver();
}

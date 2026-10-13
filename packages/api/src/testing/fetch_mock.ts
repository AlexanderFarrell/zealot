import { vi } from "vitest";
import { SetApiKey } from "@websoil/engine";

export interface RecordedRequest {
    url: string;
    method: string;
    body: unknown;
}

type Reply = { status?: number; json?: unknown; text?: string };
type Handler = (request: RecordedRequest) => Reply;

/**
 * Replaces `fetch` with a handler and records every request. Sends requests with an API key so the
 * CSRF preflight does not add an extra call, which keeps `requests` to exactly what the API made.
 */
export function mockFetch(handler: Handler = () => ({ json: {} })) {
    const requests: RecordedRequest[] = [];
    SetApiKey("test-key");
    vi.stubGlobal("fetch", vi.fn(async (url: string, init?: RequestInit) => {
        const body = typeof init?.body === "string" ? JSON.parse(init.body) : init?.body;
        const request: RecordedRequest = { url: String(url), method: init?.method ?? "GET", body };
        requests.push(request);
        const reply = handler(request);
        const status = reply.status ?? 200;
        const payload = reply.text ?? JSON.stringify(reply.json ?? {});
        return new Response(payload, { status });
    }));
    return requests;
}

export function restoreFetch() {
    vi.unstubAllGlobals();
    SetApiKey(null);
}

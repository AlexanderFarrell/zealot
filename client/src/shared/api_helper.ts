import Popups from "./popups";

const CSRF_COOKIE_NAME = "csrf_";
const CSRF_READY_ENDPOINT = "/api/health";
let csrfReadyPromise: Promise<void> | null = null;

function getCookieValue(name: string): string | undefined {
    if (typeof document === "undefined") {
        return undefined;
    }
    const cookies = document.cookie.split(";").map((c) => c.trim());
    for (const cookie of cookies) {
        if (cookie.startsWith(`${name}=`)) {
            return decodeURIComponent(cookie.slice(name.length + 1));
        }
    }
    return undefined;
}

export function withCsrf(headers: Record<string, string> = {}): Record<string, string> {
    const token = getCookieValue(CSRF_COOKIE_NAME);
    if (token) {
        return { ...headers, "X-Csrf-Token": token };
    }
    return headers;
}

async function ensureCsrfToken(): Promise<void> {
    if (getCookieValue(CSRF_COOKIE_NAME)) {
        return;
    }
    if (csrfReadyPromise) {
        return csrfReadyPromise;
    }
    csrfReadyPromise = fetch(CSRF_READY_ENDPOINT, {
        method: "GET",
        credentials: "include",
    }).then(() => undefined).finally(() => {
        csrfReadyPromise = null;
    });
    return csrfReadyPromise;
}

function needsCsrf(method: string): boolean {
    return !["GET", "HEAD", "OPTIONS"].includes(method.toUpperCase());
}

async function requestWithHandling<T>(
    url: string,
    init: RequestInit,
    on_error: string,
    parse: (response: Response) => Promise<T> | T,
): Promise<T> {
    try {
        const response = await fetch(url, {
            credentials: "include",
            ...init,
        });
        if (!response.ok) {
            const error = new Error(`${on_error} (status ${response.status})`);
            (error as Error & { response?: Response }).response = response;
            throw error;
        }
        return await parse(response);
    } catch (e) {
        console.error(e);
        Popups.add_error(on_error);
        throw e;
    }
}

async function buildRequestInit(
    method: string,
    headers: Record<string, string> = {},
    body?: BodyInit,
): Promise<RequestInit> {
    const finalHeaders = needsCsrf(method) ? withCsrf(headers) : headers;
    if (needsCsrf(method)) {
        await ensureCsrfToken();
    }
    return {
        method,
        headers: finalHeaders,
        body,
    };
}

export async function get_json(url: string, on_error: string = `Failed to get ${url}`) {
    const init = await buildRequestInit("GET", {
        "Content-Type": "application/json"
    });
    return requestWithHandling(url, init, on_error, (response) => response.json());
}

export async function get_blob(url: string, accept: string = "*/*", on_error: string = `Failed to download ${url}`) {
    const init = await buildRequestInit("GET", {
        Accept: accept
    });
    return requestWithHandling(url, init, on_error, (response) => response.blob());
}

export async function get_req(url: string, on_error: string = `Failed to request to ${url}`) {
    const init = await buildRequestInit("GET", {
        "Content-Type": "application/json"
    });
    return requestWithHandling(url, init, on_error, (response) => response);
}

export async function post_json(url: string, data: any, on_error: string = `Failed to post to ${url}`) {
    const init = await buildRequestInit(
        "POST",
        {
            "Content-Type": "application/json"
        },
        JSON.stringify(data),
    );
    return requestWithHandling(url, init, on_error, (response) => response.json());
}

export async function post_req(url: string, data: any, on_error: string = `Failed to post to ${url}`) {
    const init = await buildRequestInit(
        "POST",
        {
            "Content-Type": "application/json"
        },
        JSON.stringify(data),
    );
    return requestWithHandling(url, init, on_error, (response) => response);
}

export async function post_req_form_data(url: string, data: FormData, on_error: string = `Failed to submit form to ${url}`) {
    const init = await buildRequestInit("POST", {}, data);
    return requestWithHandling(url, init, on_error, (response) => response);
}

export async function patch_json(url: string, data: any, on_error: string = `Failed to patch ${url}`) {
    const init = await buildRequestInit(
        "PATCH",
        {
            "Content-Type": "application/json"
        },
        JSON.stringify(data),
    );
    return requestWithHandling(url, init, on_error, (response) => response.json());
}

export async function patch_req(url: string, data: any, on_error: string = `Failed to patch ${url}`) {
    const init = await buildRequestInit(
        "PATCH",
        {
            "Content-Type": "application/json"
        },
        JSON.stringify(data),
    );
    return requestWithHandling(url, init, on_error, (response) => response);
}

export async function delete_req(url: string, data: any = {}, on_error: string = `Failed to delete ${url}`) {
    const init = await buildRequestInit(
        "DELETE",
        {
            "Content-Type": "application/json"
        },
        JSON.stringify(data),
    );
    return requestWithHandling(url, init, on_error, (response) => response);
}

export class BasicAPI<T> {
    protected URL: string;

    constructor(url: string) {
        this.URL = url;
    }

    async get_all(): Promise<T[]> {
        return (await get_json(this.URL)) as T[]
    }

    async add(item: T) {
        return post_req(this.URL, item)
    }

    async update(id: number, updates: any) {
        return patch_json(`${this.URL}/${id}`, updates)
    }

    async remove(id: number) {
        return delete_req(`${this.URL}/${id}`)
    }
}

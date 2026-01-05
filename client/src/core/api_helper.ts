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

export async function get_json(url: string) {
    return (await fetch(url, {
        method: "GET",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "include",
    })).json()
}

export async function get_blob(url: string, accept: string = "*/*") {
    return (await fetch(url, {
        method: "GET",
        headers: {
            Accept: accept
        },
        credentials: "include"
    })).blob()
}

export async function get_req(url: string) {
    return fetch(url, {
        method: "GET",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "include",
    })
}

export async function post_json(url: string, data: any) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "POST",
        headers: {
            ...withCsrf({
                "Content-Type": "application/json"
            })
        },
        body: JSON.stringify(data),
        credentials: "include",
    })).json()
}

export async function post_req(url: string, data: any) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "POST",
        headers: {
            ...withCsrf({
                "Content-Type": "application/json"
            })
        },
        body: JSON.stringify(data),
        credentials: "include",
    }))
}

export async function post_req_form_data(url: string, data: FormData) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "POST",
        headers: {
            ...withCsrf()
        },
        body: data,
        credentials: "include",
    }))
}

export async function patch_json(url: string, data: any) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "PATCH",
        headers: {
            ...withCsrf({
                "Content-Type": "application/json"
            })
        },
        body: JSON.stringify(data),
        credentials: "include",
    })).json()
}

export async function patch_req(url: string, data: any) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "PATCH",
        headers: {
            ...withCsrf({
                "Content-Type": "application/json"
            })
        },
        body: JSON.stringify(data),
        credentials: "include",
    }))
}

export async function delete_req(url: string, data: any = {}) {
    await ensureCsrfToken();
    return (await fetch(url, {
        method: "DELETE",
        headers: {
            ...withCsrf({
                "Content-Type": "application/json"
            })
        },
        body: JSON.stringify(data),
        credentials: "include",
    }))
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

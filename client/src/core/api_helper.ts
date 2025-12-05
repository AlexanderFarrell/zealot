export async function get_json(url: string) {
    return (await fetch(url, {
        method: "GET",
        headers: {
            "Content-Type": "application/json"
        },
    })).json()
}

export async function post_json(url: string, data: any) {
    return (await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
    })).json()
}

export async function post_req(url: string, data: any) {
    return (await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
    }))
}

export async function patch_json(url: string, data: any) {
    return (await fetch(url, {
        method: "PATCH",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
    })).json()
}

export async function delete_req(url: string) {
    return (await fetch(url, {
        method: "DELETE",
        headers: {
            "Content-Type": "application/json"
        }
    }))
}

export class BasicAPI<T> {
    private URL: string;

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


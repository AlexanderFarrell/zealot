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
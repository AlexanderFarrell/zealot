import { afterEach, describe, expect, it } from "vitest";
import { AuthAPI } from "./auth";
import { mockFetch, restoreFetch } from "./testing/fetch_mock";

const account = { account_id: 1, username: "alex", email: "a@x", given_name: "A", surname: "F", settings: {}, has_api_key: false };

afterEach(restoreFetch);

describe("AuthAPI", () => {
    it("logs in, remembers the account, and logs out", async () => {
        const requests = mockFetch((r) => (r.url.endsWith("/auth/login") ? { json: account } : { json: {} }));
        const auth = new AuthAPI("/api");
        const result = await auth.Basic.login({ username: "alex", password: "pw" });
        expect(result.Username).toBe("alex");
        expect(auth.Account?.AccountID).toBe(1);
        expect(requests[0]?.body).toEqual({ username: "alex", password: "pw" });
        await auth.Basic.logout();
        expect(auth.Account).toBeNull();
    });

    it("rejects mismatched passwords without calling the server", async () => {
        const requests = mockFetch();
        const auth = new AuthAPI("/api");
        await expect(auth.Basic.register({
            username: "a", password: "one", confirm: "two", email: "e", given_name: "g", surname: "s",
        })).rejects.toThrow("Passwords do not match.");
        expect(requests).toHaveLength(0);
    });

    it("reports login state from the server and caches the account", async () => {
        const requests = mockFetch(() => ({ json: account }));
        const auth = new AuthAPI("/api");
        expect(await auth.IsLoggedIn()).toBe(true);
        expect(await auth.IsLoggedIn()).toBe(true);
        expect(requests).toHaveLength(1);
    });

    it("is not logged in when the server refuses", async () => {
        mockFetch(() => ({ status: 401 }));
        expect(await new AuthAPI("/api").IsLoggedIn()).toBe(false);
    });

    it("manages api keys", async () => {
        const requests = mockFetch((r) =>
            r.method === "GET" ? { json: [{ api_key_id: 4, label: "cli", created_at: "now" }] } : { json: { key: "k", api_key_id: 5, label: "x", created_at: "now" } });
        const auth = new AuthAPI("/api");
        const keys = await auth.listApiKeys();
        const created = await auth.createApiKey();
        await auth.deleteApiKey(4);
        expect(keys[0]?.Label).toBe("cli");
        expect(created.key).toBe("k");
        expect(requests[1]?.body).toEqual({ label: null });
        expect(requests[2]).toMatchObject({ method: "DELETE", url: "/api/account/api-keys/4" });
    });
});

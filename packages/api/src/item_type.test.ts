import { afterEach, describe, expect, it } from "vitest";
import { ItemTypeAPI } from "./item_type";
import { mockFetch, restoreFetch } from "./testing/fetch_mock";

const typeDto = { type_id: 1, name: "Ticket", description: "", is_system: false, required_attributes: ["Status"] };

afterEach(restoreFetch);

describe("ItemTypeAPI", () => {
    it("returns null for a missing type instead of throwing", async () => {
        mockFetch(() => ({ status: 404 }));
        const api = new ItemTypeAPI("/api");
        expect(await api.get_by_id(9)).toBeNull();
        expect(await api.get_by_name("Nope")).toBeNull();
    });

    it("indexes types by name through the lazy cache", async () => {
        const requests = mockFetch(() => ({ json: [typeDto] }));
        const api = new ItemTypeAPI("/api");
        const types = await api.Types.Get();
        await api.Types.Get();
        expect(types["Ticket"]?.RequiredAttributes).toEqual(["Status"]);
        expect(requests).toHaveLength(1);
    });

    it("refreshes the cache after a write, and only once", async () => {
        const requests = mockFetch((r) => (r.method === "GET" ? { json: [typeDto] } : { json: typeDto }));
        const api = new ItemTypeAPI("/api");
        await api.Types.Get();
        await api.create({ name: "New", description: "", required_attributes: [] });
        await api.Types.Get();
        await api.Types.Get();
        expect(requests.filter(r => r.method === "GET")).toHaveLength(2);
    });

    it("pages non-compliant items", async () => {
        const requests = mockFetch(() => ({ json: { total: 0, items: [] } }));
        await new ItemTypeAPI("/api").non_compliant(3, 10, 20);
        expect(requests[0]?.url).toBe("/api/item_type/3/non_compliant?limit=10&offset=20");
    });
});

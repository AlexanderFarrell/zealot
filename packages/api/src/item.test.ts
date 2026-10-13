import { afterEach, describe, expect, it } from "vitest";
import { ItemAPI } from "./item";
import { mockFetch, restoreFetch } from "./testing/fetch_mock";

const dto = { item_id: 1, title: "A", content: "" };
const api = new ItemAPI("/api");

afterEach(restoreFetch);

describe("ItemAPI reads", () => {
    it("lists items, optionally by type, encoding the type", async () => {
        const requests = mockFetch(() => ({ json: [dto] }));
        const items = await api.GetAll();
        await api.GetAll("My Type/1");
        expect(items[0]?.Title).toBe("A");
        expect(requests.map(r => r.url)).toEqual(["/api/item", "/api/item?type=My%20Type%2F1"]);
    });

    it("encodes titles in the path", async () => {
        const requests = mockFetch(() => ({ json: dto }));
        await api.GetByTitle("Z209 - Baseline?");
        expect(requests[0]?.url).toBe("/api/item/title/Z209%20-%20Baseline%3F");
    });

    it("builds search urls from options", async () => {
        const requests = mockFetch(() => ({ json: [{ ...dto, match_scope: "content", snippet: "…hit…" }] }));
        const results = await api.Search("a&b", { scope: "content", regex: true, limit: 5, offset: 0, includeArchived: true });
        expect(requests[0]?.url).toBe("/api/item/search?term=a%26b&scope=content&include_archived=true&regex=true&limit=5&offset=0");
        expect(results[0]?.matchScope).toBe("content");
        expect(results[0]?.snippet).toBe("…hit…");
        expect(results[0]?.item.Title).toBe("A");
    });

    it("defaults search to the title scope with no snippet", async () => {
        const requests = mockFetch(() => ({ json: [{ ...dto, match_scope: "title" }] }));
        const results = await api.Search("x");
        expect(requests[0]?.url).toBe("/api/item/search?term=x&scope=title");
        expect(results[0]?.snippet).toBeNull();
    });

    it("queries a page with defaults", async () => {
        const requests = mockFetch(() => ({ json: { items: [dto], total: 41 } }));
        const page = await api.Query({ types: ["Ticket"] });
        expect(requests[0]?.body).toEqual({ types: ["Ticket"], limit: 50, offset: 0, include_archived: false });
        expect(page.total).toBe(41);
        expect(page.items).toHaveLength(1);
    });

    it("returns an empty list when random items fail", async () => {
        mockFetch(() => ({ status: 500 }));
        expect(await api.GetRandom()).toEqual([]);
    });

    it("maps most-viewed entries to items with a count", async () => {
        mockFetch(() => ({ json: [{ item_id: 3, title: "Hot", view_count: 12 }] }));
        const entries = await api.GetMostViewed();
        expect(entries[0]?.viewCount).toBe(12);
        expect(entries[0]?.item.ItemID).toBe(3);
    });

    it("builds export urls", () => {
        expect(api.ExportPdfUrl(5)).toBe("/api/item/id/5/export/pdf");
        expect(api.ExportDocxUrl(5)).toBe("/api/item/id/5/export/docx");
    });
});

describe("ItemAPI writes", () => {
    it("posts new items", async () => {
        const requests = mockFetch(() => ({ json: dto }));
        const item = await api.Add({ title: "A", types: ["Ticket"] });
        expect(requests[0]).toMatchObject({ url: "/api/item", method: "POST", body: { title: "A", types: ["Ticket"] } });
        expect(item.ItemID).toBe(1);
    });

    it("patches updates and parses the response", async () => {
        const requests = mockFetch(() => ({ json: { ...dto, title: "B" } }));
        const item = await api.Update(1, { item_id: 1, title: "B" });
        expect(requests[0]).toMatchObject({ url: "/api/item/1", method: "PATCH" });
        expect(item.Title).toBe("B");
    });

    it("sends archive flags and unassigns types with an encoded name", async () => {
        const requests = mockFetch(() => ({ json: { item_ids: [1], archived: true } }));
        await api.SetArchived(1, true, true);
        await api.UnassignType(1, "Big Type");
        expect(requests[0]?.body).toEqual({ archived: true, subtree: true });
        expect(requests[1]).toMatchObject({ method: "DELETE", url: "/api/item/1/assign_type/Big%20Type" });
    });

    it("throws a descriptive error on a failed request", async () => {
        mockFetch(() => ({ status: 404 }));
        await expect(api.GetById(99)).rejects.toThrow(/Failed to get \/api\/item\/id\/99 \(status 404\)/);
    });

    it("imports each item independently, recording the failures", async () => {
        mockFetch((request) => {
            const body = request.body as { title: string };
            return body.title === "bad" ? { status: 400 } : { json: dto };
        });
        const results = await api.ImportItems([{ title: "ok" }, { title: "bad" }, { title: "ok2" }]);
        expect(results.map(r => r.item !== null)).toEqual([true, false, true]);
        expect(results[1]?.error).toMatch(/status 400/);
    });
});

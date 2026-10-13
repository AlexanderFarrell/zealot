import { DateTime } from "luxon";
import { afterEach, describe, expect, it } from "vitest";
import { AttributeKindAPI } from "./attribute_kind";
import ZealotAPI from "./index";
import { TimeBlockAPI } from "./time_block";
import { mockFetch, restoreFetch } from "./testing/fetch_mock";

const item = { item_id: 1, title: "A", content: "" };
const day = DateTime.fromISO("2026-10-13");
const blockDto = { block_id: 1, item, date: "2026-10-13", start_min: 540, end_min: 600, note: "" };

afterEach(restoreFetch);

describe("AttributeKindAPI", () => {
    const kindDto = { kind_id: 1, key: "Status", description: "", base_type: "dropdown", config: {}, is_system: true };

    it("uses the id and key routes the server defines", async () => {
        const requests = mockFetch(() => ({ json: {} }));
        const api = new AttributeKindAPI("/api");
        await api.update(3, { kind_id: 3, description: "d" });
        await api.removeByKey("Status");
        expect(requests[0]).toMatchObject({ method: "PATCH", url: "/api/attribute/id/3" });
        expect(requests[1]).toMatchObject({ method: "DELETE", url: "/api/attribute/key/Status" });
    });

    it("indexes kinds by key and rejects a kind with a bad base type", async () => {
        mockFetch(() => ({ json: [kindDto] }));
        const kinds = await new AttributeKindAPI("/api").Kinds.Get();
        expect(kinds["Status"]?.BaseType).toBe("dropdown");
        restoreFetch();
        mockFetch(() => ({ json: [{ ...kindDto, base_type: "bogus" }] }));
        await expect(new AttributeKindAPI("/api").Kinds.Get()).rejects.toThrow(/Invalid base type/);
    });
});

describe("TimeBlockAPI", () => {
    it("reads by day and item", async () => {
        const requests = mockFetch(() => ({ json: [blockDto] }));
        const api = new TimeBlockAPI("/api");
        const blocks = await api.GetForDay(day);
        await api.GetForItem(1);
        expect(blocks[0]?.startLabel()).toBe("09:00");
        expect(requests.map(r => r.url)).toEqual(["/api/time_block/day/2026-10-13", "/api/time_block/item/1"]);
    });

    it("creates, updates and deletes", async () => {
        const requests = mockFetch(() => ({ json: blockDto }));
        const api = new TimeBlockAPI("/api");
        const created = await api.Create({ item_id: 1, date: "2026-10-13", start_min: 540, end_min: 600 });
        await api.Update({ block_id: 1, note: "n" });
        await api.Delete(1);
        expect(created.BlockId).toBe(1);
        expect(requests.map(r => r.method)).toEqual(["POST", "PATCH", "DELETE"]);
        expect(requests[1]?.url).toBe("/api/time_block/1");
    });
});

describe("MediaAPI", () => {
    it("makes folders, renames, deletes with trimmed paths", async () => {
        const requests = mockFetch();
        const api = new ZealotAPI("/api").Media;
        await api.MakeFolder("photos/new");
        await api.Rename("photos/a.png", "b.png");
        await api.Delete("photos/old///");
        expect(requests[0]).toMatchObject({ method: "POST", url: "/api/media/mkdir", body: { folder: "photos/new" } });
        expect(requests[1]?.body).toEqual({ old_location: "photos/a.png", new_name: "b.png" });
        expect(requests[2]?.url).toBe("/api/media/photos/old");
    });

    it("uploads a file as form data to the trimmed location", async () => {
        const requests = mockFetch();
        await new ZealotAPI("/api").Media.UploadFolder(new File(["x"], "a.txt"), "docs/");
        expect(requests[0]).toMatchObject({ method: "POST", url: "/api/media/docs" });
        expect(requests[0]?.body).toBeInstanceOf(FormData);
    });
});

describe("RuleAPI", () => {
    const ruleDto = {
        rule_id: "r", account_id: "a", name: "n", description: "", trigger: { kind: "manual" }, script: "",
        enabled: true, created_at: "", last_run_at: null, last_error: null, last_output: null,
    };

    it("covers get, add, update and delete", async () => {
        const requests = mockFetch(() => ({ json: ruleDto }));
        const api = new ZealotAPI("/api").Rule;
        expect((await api.GetById("r")).rule_id).toBe("r");
        await api.Add({ name: "n", trigger: { kind: "manual" }, script: "" });
        await api.Update("r", { enabled: false });
        await api.Delete("r");
        expect(requests.map(r => `${r.method} ${r.url}`)).toEqual([
            "GET /api/rule/r", "POST /api/rule", "PATCH /api/rule/r", "DELETE /api/rule/r",
        ]);
    });
});

describe("StatisticAPI writes", () => {
    const entry = {
        statistic_entry_id: 1, item_id: 7, value: 2, occurred_at: "2026-10-13T08:00:00Z",
        related_item_id: null, comment: null, created_at: "2026-10-13T08:00:00Z", updated_at: "2026-10-13T08:00:00Z",
    };

    it("lists items with and without a parent", async () => {
        const requests = mockFetch(() => ({ json: [item] }));
        const api = new ZealotAPI("/api").Statistic;
        await api.GetItems();
        await api.GetItems(9);
        expect(requests.map(r => r.url)).toEqual(["/api/statistic/items", "/api/statistic/items?parent_id=9"]);
    });

    it("creates, updates and deletes entries", async () => {
        const requests = mockFetch(() => ({ json: entry }));
        const api = new ZealotAPI("/api").Statistic;
        expect((await api.Create(7, { value: 2 })).Value).toBe(2);
        expect((await api.Update(1, { value: 3 })).ItemID).toBe(7);
        await api.Delete(1);
        expect(requests.map(r => `${r.method} ${r.url}`)).toEqual([
            "POST /api/statistic/7/entries", "PATCH /api/statistic/entries/1", "DELETE /api/statistic/entries/1",
        ]);
    });
});

describe("ItemAPI relations", () => {
    it("reads children, related, backlinks, recent and filters", async () => {
        const requests = mockFetch(() => ({ json: [item] }));
        const api = new ZealotAPI("/api").Item;
        await api.GetChildren(1);
        await api.GetRelated(1);
        await api.GetBacklinks(1);
        await api.GetRecent(5, 10);
        await api.Filter([{ key: "Status", op: "eq", value: "Working", list_mode: "Any" }]);
        expect(requests.map(r => r.url)).toEqual([
            "/api/item/children/1", "/api/item/related/1", "/api/item/backlinks/1",
            "/api/item/recent?limit=5&offset=10", "/api/item/filter",
        ]);
        expect(requests[4]?.body).toEqual({ filters: [{ key: "Status", op: "eq", value: "Working", list_mode: "Any" }] });
    });

    it("deletes, assigns types, rebuilds links", async () => {
        const requests = mockFetch(() => ({ json: { rebuilt: 3 } }));
        const api = new ZealotAPI("/api").Item;
        await api.Delete(4);
        await api.AssignType(4, "A B");
        expect(await api.RebuildLinks()).toEqual({ rebuilt: 3 });
        expect(requests.map(r => `${r.method} ${r.url}`)).toEqual([
            "DELETE /api/item/4", "POST /api/item/4/assign_type/A%20B", "POST /api/item/rebuild-links",
        ]);
    });
});

describe("ItemTypeAPI writes", () => {
    it("assigns and unassigns attribute kinds and reads summaries", async () => {
        const requests = mockFetch((r) => (r.url.endsWith("/summary")
            ? { json: [{ type_id: 1, name: "T", is_system: false, required_attributes_count: 2, item_count: 5 }] }
            : { json: {} }));
        const api = new ZealotAPI("/api").ItemType;
        await api.assign(1, ["Status"]);
        await api.unassign(1, ["Status"]);
        await api.update(1, { type_id: 1, dry_run: true });
        await api.remove(1);
        const summaries = await api.get_summaries();
        expect(summaries[0]?.ItemCount).toBe(5);
        expect(requests.slice(0, 4).map(r => `${r.method} ${r.url}`)).toEqual([
            "POST /api/item_type/1/attr_kind", "DELETE /api/item_type/1/attr_kind",
            "PATCH /api/item_type/1", "DELETE /api/item_type/1",
        ]);
    });
});

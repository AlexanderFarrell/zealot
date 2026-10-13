import { DateTime } from "luxon";
import { afterEach, describe, expect, it } from "vitest";
import ZealotAPI from "./index";
import { mockFetch, restoreFetch } from "./testing/fetch_mock";

const item = { item_id: 1, title: "A", content: "" };
const day = DateTime.fromISO("2026-10-13");

afterEach(restoreFetch);

describe("PlannerAPI", () => {
    it("addresses day, ISO week, month and year", async () => {
        const requests = mockFetch(() => ({ json: [item] }));
        const api = new ZealotAPI("/api");
        await api.Planner.GetForDay(day);
        await api.Planner.GetForWeek(day);
        await api.Planner.GetForMonth(day);
        await api.Planner.GetForYear(day);
        expect(requests.map(r => r.url)).toEqual([
            "/api/planner/day/2026-10-13",
            "/api/planner/week/2026-W42",
            "/api/planner/month/10/year/2026",
            "/api/planner/year/2026",
        ]);
    });

    it("uses the ISO week-year at a year boundary", async () => {
        const requests = mockFetch(() => ({ json: [] }));
        await new ZealotAPI("/api").Planner.GetForWeek(DateTime.fromISO("2027-01-01"));
        expect(requests[0]?.url).toBe("/api/planner/week/2026-W53");
    });
});

describe("CommentAPI", () => {
    const commentDto = { comment_id: 2, item, timestamp: "2026-10-13 09:00:00", content: "hi" };

    it("formats the timestamp when adding", async () => {
        const requests = mockFetch(() => ({ json: commentDto }));
        const comment = await new ZealotAPI("/api").Comment.AddComment({
            item_id: 1, content: "hi", timestamp: DateTime.fromObject({ year: 2026, month: 10, day: 13, hour: 9 }),
        });
        expect(requests[0]?.body).toEqual({ content: "hi", item_id: 1, timestamp: "2026-10-13 09:00:00" });
        expect(comment.CommentID).toBe(2);
    });

    it("sends only the fields being updated", async () => {
        const requests = mockFetch(() => ({ json: commentDto }));
        await new ZealotAPI("/api").Comment.UpdateEntry({ comment_id: 2, content: "edited" });
        expect(requests[0]).toMatchObject({ method: "PATCH", url: "/api/comment/2", body: { comment_id: 2, content: "edited" } });
    });

    it("lists by day and item", async () => {
        const requests = mockFetch(() => ({ json: [commentDto] }));
        const api = new ZealotAPI("/api");
        await api.Comment.GetForDay(day);
        await api.Comment.GetForItem(1);
        expect(requests.map(r => r.url)).toEqual(["/api/comment/day/2026-10-13", "/api/comment/item/1"]);
    });
});

describe("RepeatAPI and TimeBlock-style date payloads", () => {
    it("puts the ISO date in a status change", async () => {
        const requests = mockFetch();
        await new ZealotAPI("/api").Repeat.SetStatus({ item_id: 1, date: day, status: "Complete" });
        expect(requests[0]).toMatchObject({ method: "PUT", url: "/api/repeat/status" });
        expect(requests[0]?.body).toMatchObject({ item_id: 1, date: "2026-10-13", status: "Complete" });
    });
});

describe("AttributeAPI", () => {
    it("sets, renames and removes attributes", async () => {
        const requests = mockFetch();
        const api = new ZealotAPI("/api").Attribute;
        await api.set_value(1, "Status", "Working");
        await api.rename(1, "Old", "New");
        await api.remove(1, "Status");
        expect(requests[0]).toMatchObject({ method: "PATCH", url: "/api/item/1/attr", body: { Status: "Working" } });
        expect(requests[1]?.body).toEqual({ old_key: "Old", new_key: "New" });
        expect(requests[2]).toMatchObject({ method: "DELETE", url: "/api/item/1/attr/Status" });
    });
});

describe("MediaAPI", () => {
    it("trims trailing slashes and paginates listings", async () => {
        const requests = mockFetch(() => ({ json: { files: [{ path: "a.png", size: 1, is_folder: false, modified_at: 0 }] } }));
        const api = new ZealotAPI("/api").Media;
        const root = await api.ListFiles("", 2);
        await api.ListFiles("photos/2026//");
        expect(requests.map(r => r.url)).toEqual(["/api/media?page=2", "/api/media/photos/2026?page=1"]);
        expect(root[0]?.Name).toBe("a.png");
    });
});

describe("RuleAPI", () => {
    const ruleDto = {
        rule_id: "r", account_id: "a", name: "n", description: "", trigger: { kind: "manual" }, script: "",
        enabled: true, created_at: "", last_run_at: null, last_error: null, last_output: null,
    };

    it("loads rules", async () => {
        mockFetch(() => ({ json: [ruleDto] }));
        expect((await new ZealotAPI("/api").Rule.GetAll())[0]?.name).toBe("n");
    });

    it("returns the parsed run result and throws on a failed request", async () => {
        const result = { rule_id: "r", success: true, output: "ok", error: null, duration_ms: 3 };
        mockFetch(() => ({ json: result }));
        expect(await new ZealotAPI("/api").Rule.Run("r")).toEqual(result);
        restoreFetch();
        mockFetch(() => ({ status: 500 }));
        await expect(new ZealotAPI("/api").Rule.Run("r")).rejects.toThrow(/status 500/);
    });
});

describe("StatisticAPI", () => {
    it("builds range queries only when bounds are given", async () => {
        const requests = mockFetch(() => ({ json: [] }));
        const api = new ZealotAPI("/api").Statistic;
        await api.GetDaily(7);
        await api.GetDaily(7, "2026-10-01", "2026-10-13");
        await api.GetSummary(7, "2026-10-01");
        expect(requests.map(r => r.url)).toEqual([
            "/api/statistic/7/daily",
            "/api/statistic/7/daily?start=2026-10-01&end=2026-10-13",
            "/api/statistic/7/summary?start=2026-10-01",
        ]);
    });

    it("pages entries with defaults and maps next_offset", async () => {
        const requests = mockFetch(() => ({ json: { count: 1, next_offset: 50, entries: [] } }));
        const page = await new ZealotAPI("/api").Statistic.GetEntries(7);
        expect(requests[0]?.url).toBe("/api/statistic/7/entries?limit=50&offset=0");
        expect(page.nextOffset).toBe(50);
    });
});

describe("TimeBlockAPI", () => {
    it("is exported through its module and addresses ranges by ISO date", async () => {
        const { TimeBlockAPI } = await import("./time_block");
        const requests = mockFetch(() => ({ json: [] }));
        await new TimeBlockAPI("/api").GetForRange(day, day.plus({ days: 6 }));
        expect(requests[0]?.url).toBe("/api/time_block/range?start=2026-10-13&end=2026-10-19");
    });
});

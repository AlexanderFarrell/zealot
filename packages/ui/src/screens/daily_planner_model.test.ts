import { describe, expect, it } from "vitest";
import { DateTime } from "luxon";
import { Item } from "@zealot/domain/src/item";
import {
    NO_VALUE_GROUP,
    OTHER_GROUP,
    carryOverWindow,
    chipLabels,
    dayScore,
    groupItems,
    isCarriedOver,
    isHeaderItem,
    itemPoints,
    sortByPoints,
} from "./daily_planner_model";

let nextId = 1;
function item(attributes: Record<string, unknown> = {}, types: string[] = []): Item {
    const id = nextId++;
    return new Item({
        item_id: id,
        title: `item ${id}`,
        content: "",
        attributes,
        types: types.map((name, i) => ({ type_id: i + 1, name, is_system: false })),
    });
}

const day = DateTime.fromISO("2026-11-07").startOf("day");
const PA = ["Priority", "AP"];

describe("itemPoints", () => {
    it("multiplies 100 by the points attributes", () => {
        expect(itemPoints(item({ Priority: 8, AP: 3 }), PA)).toBe(2400);
    });

    it("treats missing values as 1", () => {
        expect(itemPoints(item(), PA)).toBe(100);
        expect(itemPoints(item({ Priority: 5 }), PA)).toBe(500);
        expect(itemPoints(item({ AP: 2 }), PA)).toBe(200);
    });

    it("treats blank, null and non-numeric values as missing", () => {
        expect(itemPoints(item({ Priority: "", AP: null }), PA)).toBe(100);
        expect(itemPoints(item({ Priority: "high" }), PA)).toBe(100);
    });

    it("reads numeric strings and keeps an explicit zero", () => {
        expect(itemPoints(item({ Priority: "4", AP: "2" }), PA)).toBe(800);
        expect(itemPoints(item({ Priority: 0 }), PA)).toBe(0);
    });

    it("uses whichever attributes are configured", () => {
        expect(itemPoints(item({ Priority: 8, Effort: 5 }), ["Effort"])).toBe(500);
        expect(itemPoints(item({ Priority: 8 }), [])).toBe(100);
    });
});

describe("sortByPoints", () => {
    it("sorts highest first and keeps ties in order", () => {
        const low = item({ Priority: 2 });
        const tieA = item({ Priority: 5 });
        const tieB = item({ Priority: 5 });
        const high = item({ Priority: 3, AP: 3 });
        expect(sortByPoints([low, tieA, tieB, high], PA)).toEqual([high, tieA, tieB, low]);
    });

    it("keeps server order when points are off", () => {
        const a = item({ Priority: 1 });
        const b = item({ Priority: 9 });
        expect(sortByPoints([a, b], [])).toEqual([a, b]);
    });

    it("does not mutate its input", () => {
        const input = [item({ Priority: 1 }), item({ Priority: 9 })];
        const copy = [...input];
        sortByPoints(input, PA);
        expect(input).toEqual(copy);
    });
});

describe("groupItems", () => {
    const executor = { key: "Executor", values: ["Alexander", "Mixed", "LLM", "External"] };
    const work = (attributes: Record<string, unknown> = {}): Item => item({ Status: "To Do", ...attributes });

    it("orders groups by the dropdown's values and sorts each by points", () => {
        const llm = work({ Executor: "LLM", Priority: 2 });
        const meLow = work({ Executor: "Alexander", Priority: 1 });
        const meHigh = work({ Executor: "Alexander", Priority: 9 });
        const ext = work({ Executor: "External" });
        const groups = groupItems([llm, meLow, ext, meHigh], executor, PA);
        expect(groups.map((g) => g.label)).toEqual(["Alexander", "LLM", "External"]);
        expect(groups[0]!.items).toEqual([meHigh, meLow]);
    });

    it("hides empty groups", () => {
        const groups = groupItems([work({ Executor: "Mixed" })], executor, PA);
        expect(groups.map((g) => g.label)).toEqual(["Mixed"]);
    });

    it("puts unknown values after known ones and items without a value last", () => {
        const none = work();
        const blank = work({ Executor: "" });
        const unknownA = work({ Executor: "Contractor" });
        const unknownB = work({ Executor: "Agency" });
        const me = work({ Executor: "Alexander" });
        const groups = groupItems([none, unknownA, blank, me, unknownB], executor, PA);
        expect(groups.map((g) => g.label)).toEqual(["Alexander", "Contractor", "Agency", "No Executor"]);
        expect(groups[3]!.key).toBe(NO_VALUE_GROUP);
        expect(groups[3]!.items).toEqual([none, blank]);
    });

    it("uses the old planner's fixed order for Status, Rejected and Retired last", () => {
        const statuses = ["Complete", "Retired", "To Do", "Waiting", "Rejected", "Working", "Blocked", "Specify", "Hold", "On Going"];
        const items = statuses.map((Status) => item({ Status }));
        const groups = groupItems(items, { key: "Status", values: ["To Do", "Specify", "Working"] }, []);
        expect(groups.map((g) => g.label)).toEqual([
            "Working", "Specify", "To Do", "On Going", "Hold", "Blocked", "Complete", "Waiting", "Rejected", "Retired",
        ]);
    });

    it("returns one unlabelled group when not grouping, and nothing for an empty list", () => {
        const a = work({ Priority: 1 });
        const b = work({ Priority: 5 });
        const groups = groupItems([a, b], null, PA);
        expect(groups).toHaveLength(1);
        expect(groups[0]!.label).toBe("");
        expect(groups[0]!.items).toEqual([b, a]);
        expect(groupItems([], null, PA)).toEqual([]);
    });

    it("puts items without a Status in a last Other group, after No <Attribute>", () => {
        const note = item({ Executor: "Alexander", Priority: 9 });
        const journal = item({ Status: "" });
        const unassigned = work();
        const mine = work({ Executor: "Alexander" });
        const groups = groupItems([note, unassigned, journal, mine], executor, PA);
        expect(groups.map((g) => g.label)).toEqual(["Alexander", "No Executor", "Other"]);
        expect(groups[0]!.items).toEqual([mine]);
        expect(groups[2]!.key).toBe(OTHER_GROUP);
        expect(groups[2]!.items).toEqual([note, journal]);
    });

    it("puts Other after the unlabelled group when not grouping, and after Status groups", () => {
        const note = item();
        const task = work();
        expect(groupItems([note, task], null, PA).map((g) => g.label)).toEqual(["", "Other"]);
        expect(groupItems([note], null, PA).map((g) => g.label)).toEqual(["Other"]);
        const byStatus = groupItems([note, task], { key: "Status", values: [] }, PA);
        expect(byStatus.map((g) => g.label)).toEqual(["To Do", "Other"]);
    });
});

describe("isHeaderItem", () => {
    it("matches the configured types only", () => {
        expect(isHeaderItem(item({}, ["Log"]), ["Log"])).toBe(true);
        expect(isHeaderItem(item({}, ["Plan"]), ["Log"])).toBe(false);
        expect(isHeaderItem(item({}, ["Log"]), [])).toBe(false);
    });
});

describe("carried over", () => {
    it("spans the 14 days before the shown day", () => {
        expect(carryOverWindow(day)).toEqual({ from: "2026-10-24", to: "2026-11-06" });
    });

    it("includes yesterday and the 14th day back, excludes the shown day and the 15th", () => {
        expect(isCarriedOver(item({ Date: "2026-11-06", Status: "To Do" }), day, [])).toBe(true);
        expect(isCarriedOver(item({ Date: "2026-10-24", Status: "To Do" }), day, [])).toBe(true);
        expect(isCarriedOver(item({ Date: "2026-11-07", Status: "To Do" }), day, [])).toBe(false);
        expect(isCarriedOver(item({ Date: "2026-10-23", Status: "To Do" }), day, [])).toBe(false);
        expect(isCarriedOver(item({ Date: "2026-11-08", Status: "To Do" }), day, [])).toBe(false);
    });

    it("crosses month and year boundaries", () => {
        const newYear = DateTime.fromISO("2027-01-01").startOf("day");
        expect(isCarriedOver(item({ Date: "2026-12-31", Status: "To Do" }), newYear, [])).toBe(true);
        expect(isCarriedOver(item({ Date: "2026-12-18", Status: "To Do" }), newYear, [])).toBe(true);
        expect(isCarriedOver(item({ Date: "2026-12-17", Status: "To Do" }), newYear, [])).toBe(false);
    });

    it("excludes Complete, Rejected, Retired and Hold", () => {
        for (const status of ["Complete", "Rejected", "Retired", "Hold"]) {
            expect(isCarriedOver(item({ Date: "2026-11-05", Status: status }), day, [])).toBe(false);
        }
    });

    it("keeps other statuses and every Executor", () => {
        for (const status of ["To Do", "Working", "Specify", "Blocked", "On Going"]) {
            expect(isCarriedOver(item({ Date: "2026-11-05", Status: status }), day, [])).toBe(true);
        }
        expect(isCarriedOver(item({ Date: "2026-11-05", Status: "To Do", Executor: "LLM" }), day, [])).toBe(true);
    });

    it("never carries items without a Status (notes, journal entries)", () => {
        expect(isCarriedOver(item({ Date: "2026-11-05" }), day, [])).toBe(false);
        expect(isCarriedOver(item({ Date: "2026-11-05", Status: "" }), day, [])).toBe(false);
    });

    it("skips header items and items without a Date", () => {
        expect(isCarriedOver(item({ Date: "2026-11-05", Status: "To Do" }, ["Log"]), day, ["Log"])).toBe(false);
        expect(isCarriedOver(item({ Date: "2026-11-05", Status: "To Do" }, ["Log"]), day, [])).toBe(true);
        expect(isCarriedOver(item({ Status: "To Do" }), day, [])).toBe(false);
    });
});

describe("dayScore", () => {
    it("earns Complete items and plans today plus carried over", () => {
        const done = item({ Priority: 8, Status: "Complete" });
        const open = item({ Priority: 5, Status: "Working" });
        const carried = item({ Priority: 3, Status: "To Do", Date: "2026-11-06" });
        expect(dayScore([done, open], [carried], PA)).toEqual({ earned: 800, planned: 1600 });
    });

    it("leaves out items without a Status", () => {
        const done = item({ Priority: 8, Status: "Complete" });
        const note = item({ Priority: 9 });
        expect(dayScore([done, note], [], PA)).toEqual({ earned: 800, planned: 800 });
    });

    it("is zero for an empty day", () => {
        expect(dayScore([], [], PA)).toEqual({ earned: 0, planned: 0 });
    });
});

describe("chipLabels", () => {
    it("shortens long names to a letter and keeps short ones", () => {
        expect(chipLabels(["Priority", "AP"])).toEqual({ Priority: "P", AP: "AP" });
    });

    it("falls back to full names when first letters clash", () => {
        expect(chipLabels(["Priority", "Points", "Effort"])).toEqual({ Priority: "Priority", Points: "Points", Effort: "E" });
    });
});


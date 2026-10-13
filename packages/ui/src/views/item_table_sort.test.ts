import { describe, expect, it } from "vitest";
import { AttributeKind } from "@zealot/domain/src/attribute";
import { Item } from "@zealot/domain/src/item";
import { keyForColumn, sortItems, toggleSort } from "./item_table_sort";
import type { ItemTableColumn } from "./item_table_types";

const title: ItemTableColumn = { kind: "title" };
const priority: ItemTableColumn = { kind: "attribute", attributeKey: "Priority" };
const kinds = {
    Priority: new AttributeKind({ kind_id: 1, key: "Priority", description: "", base_type: "integer", config: {}, is_system: false }),
};

function item(id: number, name: string, attributes: Record<string, unknown> = {}): Item {
    return new Item({ item_id: id, title: name, content: "", attributes });
}

describe("keyForColumn", () => {
    it("namespaces attribute columns", () => {
        expect(keyForColumn(title)).toBe("title");
        expect(keyForColumn(priority)).toBe("attr:Priority");
    });
});

describe("toggleSort", () => {
    it("cycles asc, desc, off for one column", () => {
        const first = toggleSort(null, null, title);
        expect(first).toEqual({ key: "title", dir: "asc" });
        const second = toggleSort(first.key, first.dir, title);
        expect(second).toEqual({ key: "title", dir: "desc" });
        expect(toggleSort(second.key, second.dir, title)).toEqual({ key: null, dir: null });
    });

    it("starts ascending when switching column", () => {
        expect(toggleSort("title", "desc", priority)).toEqual({ key: "attr:Priority", dir: "asc" });
    });
});

describe("sortItems", () => {
    const items = [item(1, "banana", { Priority: 10 }), item(2, "Apple", { Priority: 2 }), item(3, "cherry")];

    it("does not mutate or reorder without a sort key", () => {
        expect(sortItems(items, null, "asc", [title], kinds).map(i => i.ItemID)).toEqual([1, 2, 3]);
        expect(sortItems(items, "title", null, [title], kinds).map(i => i.ItemID)).toEqual([1, 2, 3]);
        expect(sortItems(items, "missing", "asc", [title], kinds).map(i => i.ItemID)).toEqual([1, 2, 3]);
    });

    it("sorts titles case-insensitively", () => {
        expect(sortItems(items, "title", "asc", [title], kinds).map(i => i.Title)).toEqual(["Apple", "banana", "cherry"]);
        expect(sortItems(items, "title", "desc", [title], kinds).map(i => i.Title)).toEqual(["cherry", "banana", "Apple"]);
    });

    it("sorts numbers numerically, not as text", () => {
        expect(sortItems(items, "attr:Priority", "asc", [priority], kinds).map(i => i.ItemID)).toEqual([2, 1, 3]);
    });

    it("keeps items with no value last when ascending", () => {
        const ids = sortItems(items, "attr:Priority", "asc", [priority], kinds).map(i => i.ItemID);
        expect(ids[ids.length - 1]).toBe(3);
    });

    it("sorts by joined type names", () => {
        const typed = [
            new Item({ item_id: 1, title: "x", content: "", types: [{ type_id: 1, name: "Zed", is_system: false }] }),
            new Item({ item_id: 2, title: "y", content: "", types: [{ type_id: 2, name: "Alpha", is_system: false }] }),
        ];
        expect(sortItems(typed, "types", "asc", [{ kind: "types" }], kinds).map(i => i.ItemID)).toEqual([2, 1]);
    });
});

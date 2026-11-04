import { describe, expect, it } from "vitest";
import { Item, ItemLink, type ItemDto } from "./item";

const base: ItemDto = { item_id: 1, title: "Write tests", content: "body" };

describe("Item", () => {
    it("defaults missing attributes, types and links", () => {
        const item = new Item(base);
        expect(item.Attributes).toEqual({});
        expect(item.Types).toEqual([]);
        expect(item.Links).toEqual([]);
    });

    it("maps created_at and updated_at, defaulting to 0 from older servers", () => {
        expect(new Item(base).CreatedAt).toBe(0);
        expect(new Item(base).UpdatedAt).toBe(0);
        const item = new Item({ ...base, created_at: 1_767_225_600, updated_at: 1_767_268_800 });
        expect(item.CreatedAt).toBe(1_767_225_600);
        expect(item.UpdatedAt).toBe(1_767_268_800);
    });

    it("maps types and links", () => {
        const item = new Item({
            ...base,
            types: [{ type_id: 3, name: "Ticket", is_system: false }],
            links: [{ other_item_id: 9, relationship: "parent" }],
        });
        expect(item.Types[0]?.Name).toBe("Ticket");
        expect(item.Links[0]).toBeInstanceOf(ItemLink);
        expect(item.Links[0]?.OtherItemID).toBe(9);
        expect(item.Links[0]?.Relationship).toBe("parent");
    });

    it("is archived only when the Archived attribute is exactly true", () => {
        expect(new Item(base).Archived).toBe(false);
        expect(new Item({ ...base, attributes: { Archived: true } }).Archived).toBe(true);
        expect(new Item({ ...base, attributes: { Archived: "true" } }).Archived).toBe(false);
    });

    it("builds the display title from the icon attribute", () => {
        expect(new Item(base).DisplayTitle).toBe("Write tests");
        expect(new Item(base).RawIcon).toBe("");
        const iconed = new Item({ ...base, attributes: { Icon: "🧪" } });
        expect(iconed.DisplayTitle).toBe("🧪 Write tests");
        expect(iconed.RawIcon).toBe("🧪");
    });
});

import { describe, expect, it } from "vitest";
import { effectiveDefault, ItemType, ItemTypeSummary, type ItemTypeDto } from "./item_type";

function type(name: string, required: string[], defaults: Record<string, unknown> = {}): ItemType {
    const dto: ItemTypeDto = {
        type_id: 1,
        name,
        description: "",
        is_system: false,
        required_attributes: required,
        required_defaults: defaults,
    };
    return new ItemType(dto);
}

describe("ItemType", () => {
    it("defaults required_defaults to an empty object", () => {
        const t = new ItemType({ type_id: 1, name: "A", description: "", is_system: true, required_attributes: [] });
        expect(t.RequiredDefaults).toEqual({});
        expect(t.IsSystem).toBe(true);
    });
});

describe("ItemTypeSummary", () => {
    it("maps counts", () => {
        const s = new ItemTypeSummary({ type_id: 2, name: "B", is_system: false, required_attributes_count: 3, item_count: 12 });
        expect(s.RequiredAttributesCount).toBe(3);
        expect(s.ItemCount).toBe(12);
    });
});

describe("effectiveDefault", () => {
    it("uses the kind default when no type sets one", () => {
        expect(effectiveDefault([], "Status", "To Do")).toBe("To Do");
        expect(effectiveDefault([type("A", ["Status"])], "Status", "To Do")).toBe("To Do");
    });

    it("uses the type default when one type sets it", () => {
        expect(effectiveDefault([type("A", ["Status"], { Status: "Working" })], "Status", "To Do")).toBe("Working");
    });

    it("uses the shared value when all types agree, including structured values", () => {
        const a = type("A", ["Tags"], { Tags: ["x", "y"] });
        const b = type("B", ["Tags"], { Tags: ["x", "y"] });
        expect(effectiveDefault([a, b], "Tags", [])).toEqual(["x", "y"]);
    });

    it("falls back to the kind default when types disagree", () => {
        const a = type("A", ["Status"], { Status: "Working" });
        const b = type("B", ["Status"], { Status: "Hold" });
        expect(effectiveDefault([a, b], "Status", "To Do")).toBe("To Do");
    });

    it("ignores defaults for attributes the type does not require", () => {
        expect(effectiveDefault([type("A", [], { Status: "Working" })], "Status", "To Do")).toBe("To Do");
    });
});

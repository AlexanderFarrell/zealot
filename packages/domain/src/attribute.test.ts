import { describe, expect, it } from "vitest";
import { AttributeBaseTypesArray, AttributeKind, AttributeOpsArray, type AttributeKindDto } from "./attribute";

const dto: AttributeKindDto = {
    kind_id: 7,
    key: "Priority",
    description: "How urgent",
    base_type: "integer",
    config: { min: 1, max: 10 },
    is_system: false,
};

describe("AttributeKind", () => {
    it("maps dto fields", () => {
        const kind = new AttributeKind(dto);
        expect(kind.KindID).toBe(7);
        expect(kind.Key).toBe("Priority");
        expect(kind.BaseType).toBe("integer");
        expect(kind.Config).toEqual({ min: 1, max: 10 });
        expect(kind.IsSystem).toBe(false);
    });

    it("rejects an unknown base type", () => {
        expect(() => new AttributeKind({ ...dto, base_type: "nonsense" })).toThrow(/Invalid base type.*nonsense/);
    });

    it("accepts every declared base type", () => {
        for (const base_type of AttributeBaseTypesArray) {
            expect(new AttributeKind({ ...dto, base_type }).BaseType).toBe(base_type);
        }
    });

    it("lists the comparison operators the API understands", () => {
        expect(AttributeOpsArray).toEqual(expect.arrayContaining(["eq", "ne", "gt", "lt", "gte", "lte", "ilike"]));
    });
});

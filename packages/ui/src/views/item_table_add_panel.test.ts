import { describe, expect, it, vi } from "vitest";

vi.mock("@zealot/api/src/item_type", () => ({
    ItemTypeAPI: class {
        async get_summaries() { return [{ Name: "Plan" }, { Name: "Ticket" }]; }
        async get_by_name(name: string) { return { Name: name, RequiredAttributes: [] }; }
    },
}));

import { buildAddPanel } from "./item_table_add_panel";

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("buildAddPanel", () => {
    it("preselects the kept type when reopened after a create", async () => {
        const draft = { attributes: {}, title: "", types: ["Plan"] };
        const panel = buildAddPanel({ enabled: true }, draft, {}, true, () => {}, () => {});
        document.body.appendChild(panel);
        await flush();
        await flush();
        const select = panel.querySelector("select") as HTMLSelectElement;
        expect(select.value).toBe("Plan");
        expect(draft.types).toEqual(["Plan"]);
    });
});

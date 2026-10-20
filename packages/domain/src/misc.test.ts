import { DateTime } from "luxon";
import { describe, expect, it } from "vitest";
import { Account } from "./account";
import { RepeatEntry } from "./repeat";
import { Rule, TRIGGER_KIND_LABELS } from "./rule";
import { StatisticEntry } from "./statistic";
import { TimeBlock } from "./time_block";
import * as domain from "./main";

const item = { item_id: 1, title: "Habit", content: "" };

describe("Account", () => {
    const account = new Account({
        account_id: 1, username: "alex", email: "a@example.com", given_name: "Alexander",
        surname: "Farrell", settings: {}, has_api_key: true,
    });

    it("formats names", () => {
        expect(account.FullNameEng).toBe("Alexander Farrell");
        expect(account.FullNameFormalEng).toBe("Farrell, Alexander");
        expect(account.HasApiKey).toBe(true);
    });
});

describe("RepeatEntry", () => {
    it("parses the date and builds the item", () => {
        const entry = new RepeatEntry({ status: "Complete", item, date: "2026-10-13", comment: "ok" });
        expect(entry.Date.toISODate()).toBe("2026-10-13");
        expect(entry.Item.Title).toBe("Habit");
        expect(entry.Status).toBe("Complete");
    });

    it("yields an invalid date for a bad string instead of throwing", () => {
        expect(new RepeatEntry({ status: "Skip", item, date: "nope", comment: "" }).Date.isValid).toBe(false);
    });
});

describe("TimeBlock", () => {
    it("labels minutes as zero-padded clock times", () => {
        const block = new TimeBlock({ block_id: 1, item, date: "2026-10-13", start_min: 9 * 60 + 5, end_min: 13 * 60, note: "" });
        expect(block.startLabel()).toBe("09:05");
        expect(block.endLabel()).toBe("13:00");
        expect(new TimeBlock({ block_id: 2, item, date: "2026-10-13", start_min: 0, end_min: 1439, note: "" }).endLabel()).toBe("23:59");
    });
});

describe("StatisticEntry", () => {
    it("keeps the zone offset of timestamps", () => {
        const entry = new StatisticEntry({
            statistic_entry_id: 1, item_id: 2, value: 3.5, occurred_at: "2026-10-13T08:00:00-05:00",
            related_item_id: null, comment: null, created_at: "2026-10-13T13:00:00Z", updated_at: "2026-10-13T13:00:00Z",
        });
        expect(entry.OccurredAt.offset).toBe(-300);
        expect(entry.Value).toBe(3.5);
        expect(entry.RelatedItemID).toBeNull();
        expect(entry.CreatedAt).toBeInstanceOf(DateTime);
    });
});

describe("Rule", () => {
    it("copies every dto field", () => {
        const rule = Rule.fromDto({
            rule_id: "r1", account_id: "a1", name: "N", description: "D", trigger: { kind: "cron", expression: "0 * * * *" },
            script: "return 1", enabled: true, created_at: "x", last_run_at: null, last_error: "boom", last_output: null,
        });
        expect(rule.trigger).toEqual({ kind: "cron", expression: "0 * * * *" });
        expect(rule.last_error).toBe("boom");
        expect(rule.enabled).toBe(true);
    });

    it("labels every trigger kind", () => {
        expect(Object.keys(TRIGGER_KIND_LABELS)).toHaveLength(10);
        expect(TRIGGER_KIND_LABELS.manual).toBe("Manual");
    });
});

describe("domain barrel", () => {
    it("re-exports each module namespace", () => {
        expect(Object.keys(domain).sort()).toEqual([
            "account", "attribute", "comment", "item", "item_type", "media", "repeat", "rule", "saved_view", "statistic", "time_block",
        ]);
    });
});

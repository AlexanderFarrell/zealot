/**
 * Daily planner preferences (Z277), read from and written to the account's
 * settings so they follow the user to every device.
 */
import { AppSettings } from '@websoil/engine';

/** What the daily planner needs from settings. */
export interface DailyPlannerPreferences {
    /** The dropdown attribute to group by, or null for one list. */
    groupBy: string | null;
    pointsFrom: string[];
    headerTypes: string[];
    /** Attributes that edit inline on cards; Status means the card's Status select. */
    cardFields: string[];
}

/** Reads the daily planner preferences, tolerating missing or malformed values. */
export function readDailyPlannerPreferences(): DailyPlannerPreferences {
    const settings = AppSettings.get();
    const strings = (value: unknown): string[] =>
        Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string' && v !== '') : [];
    const groupBy = typeof settings.planner_group_by === 'string' && settings.planner_group_by !== ''
        ? settings.planner_group_by
        : null;
    return {
        groupBy,
        pointsFrom: strings(settings.planner_points_from),
        headerTypes: strings(settings.planner_header_types),
        // An account that never saved the setting gets the default; an empty list is a choice.
        cardFields: Array.isArray(settings.planner_card_fields) ? strings(settings.planner_card_fields) : ['Status'],
    };
}

/** True when the group `value` of attribute `groupBy` is collapsed. */
export function isGroupCollapsed(groupBy: string, value: string): boolean {
    const collapsed = AppSettings.get().planner_collapsed_groups;
    const values = collapsed && typeof collapsed === 'object' ? collapsed[groupBy] : undefined;
    return Array.isArray(values) && values.includes(value);
}

/** Saves whether the group `value` of attribute `groupBy` is collapsed. */
export function setGroupCollapsed(groupBy: string, value: string, collapsed: boolean): void {
    const current = AppSettings.get().planner_collapsed_groups ?? {};
    const values = new Set(Array.isArray(current[groupBy]) ? current[groupBy] : []);
    if (collapsed) values.add(value);
    else values.delete(value);
    AppSettings.set({ planner_collapsed_groups: { ...current, [groupBy]: [...values] } });
}

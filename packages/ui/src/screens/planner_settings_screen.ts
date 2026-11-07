import { AppSettings, BaseElementEmpty } from '@websoil/engine';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import { buildSectionHeader } from '../common/settings_layout';
import { LoadingSpinner } from '../common/loading_spinner';
import { loadAttributeKinds } from '../views/attribute_value_input';
import { readDailyPlannerPreferences } from './planner_preferences';

const itemTypeApi = new ItemTypeAPI('/api');

/**
 * Planner settings (Z277): how the daily planner groups, scores and trims its
 * list. Saved to the account, so they apply on every device.
 */
export class PlannerSettingsScreen extends BaseElementEmpty {
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        this.className = 'planner-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'settings-section-shell';
        const spinner = new LoadingSpinner();
        shell.append(
            buildSectionHeader('Planner', 'How the daily planner groups, scores and trims its list. Applies on every device.').element,
            spinner,
        );
        this.appendChild(shell);

        const [kinds, types] = await Promise.all([
            loadAttributeKinds(),
            itemTypeApi.Types.Get().catch(() => ({} as Record<string, unknown>)),
        ]);
        if (renderId !== this.renderId) return;

        spinner.remove();
        shell.appendChild(this._buildDailySection(Object.values(kinds), Object.keys(types).sort()));
    }

    private _buildDailySection(kinds: AttributeKind[], typeNames: string[]): HTMLElement {
        const prefs = readDailyPlannerPreferences();
        const section = document.createElement('div');
        section.className = 'preferences-section planner-preferences';

        const heading = document.createElement('h3');
        heading.textContent = 'Daily planner';
        section.appendChild(heading);

        const dropdowns = kinds.filter((k) => k.BaseType === 'dropdown').map((k) => k.Key).sort();
        const numbers = kinds.filter((k) => k.BaseType === 'integer' || k.BaseType === 'decimal').map((k) => k.Key).sort();

        section.append(
            buildSelectField(
                'Group items by',
                [{ value: '', text: 'No grouping' }, ...dropdowns.map((key) => ({ value: key, text: key }))],
                prefs.groupBy ?? '',
                (value) => AppSettings.set({ planner_group_by: value }),
                'A dropdown attribute. Groups follow its value order, set in Attributes; Status keeps the planner\'s own order.',
            ),
            buildChecklistField(
                'Points from',
                numbers,
                prefs.pointsFrom,
                (values) => AppSettings.set({ planner_points_from: values }),
                'Points are 100 × these attributes, a missing value counting as 1. None turns points and the day score off.',
            ),
            buildChecklistField(
                'Edit on cards',
                [...dropdowns, ...numbers].sort(),
                prefs.cardFields,
                (values) => AppSettings.set({ planner_card_fields: values }),
                'Fields you can change straight from a card: dropdowns as a pick list, numbers as an input. Status is the select in the card\'s corner. Items without a Status get no chips.',
            ),
            buildChecklistField(
                'Show in the header',
                typeNames,
                prefs.headerTypes,
                (values) => AppSettings.set({ planner_header_types: values }),
                'Items of these types appear as links under the date instead of as cards.',
            ),
        );
        return section;
    }
}

/** A labelled select that saves on change. */
function buildSelectField(
    labelText: string,
    options: Array<{ value: string; text: string }>,
    current: string,
    onChange: (value: string) => void,
    help: string,
): HTMLElement {
    const field = document.createElement('div');
    field.className = 'preference-field';
    const label = document.createElement('label');
    const name = document.createElement('span');
    name.textContent = labelText;
    const select = document.createElement('select');
    for (const { value, text } of options) {
        const option = document.createElement('option');
        option.value = value;
        option.textContent = text;
        select.appendChild(option);
    }
    // A saved attribute that no longer exists still shows, so it can be changed.
    if (current && !options.some((o) => o.value === current)) {
        const option = document.createElement('option');
        option.value = current;
        option.textContent = `${current} (missing)`;
        select.appendChild(option);
    }
    select.value = current;
    select.addEventListener('change', () => onChange(select.value));
    label.append(name, select);
    field.append(label, buildHelp(help));
    return field;
}

/** A labelled set of checkboxes that saves the checked values, in list order, on change. */
function buildChecklistField(
    labelText: string,
    choices: string[],
    current: string[],
    onChange: (values: string[]) => void,
    help: string,
): HTMLElement {
    const field = document.createElement('div');
    field.className = 'preference-field';
    const group = document.createElement('fieldset');
    group.className = 'preference-checklist';
    const legend = document.createElement('legend');
    legend.textContent = labelText;
    group.appendChild(legend);

    // Saved values come first, in their saved order, so the points formula keeps its order.
    const ordered = [...current, ...choices.filter((c) => !current.includes(c))];
    const boxes: HTMLInputElement[] = [];
    if (ordered.length === 0) {
        const empty = document.createElement('span');
        empty.className = 'tool-muted';
        empty.textContent = 'Nothing to choose from yet.';
        group.appendChild(empty);
    }
    for (const value of ordered) {
        const option = document.createElement('label');
        option.className = 'preference-checklist__option';
        const box = document.createElement('input');
        box.type = 'checkbox';
        box.value = value;
        box.checked = current.includes(value);
        box.addEventListener('change', () => onChange(boxes.filter((b) => b.checked).map((b) => b.value)));
        boxes.push(box);
        const text = document.createElement('span');
        text.textContent = choices.includes(value) ? value : `${value} (missing)`;
        option.append(box, text);
        group.appendChild(option);
    }
    field.append(group, buildHelp(help));
    return field;
}

function buildHelp(text: string): HTMLElement {
    const note = document.createElement('p');
    note.className = 'tool-muted';
    note.textContent = text;
    return note;
}

if (!customElements.get('planner-settings-screen')) {
    customElements.define('planner-settings-screen', PlannerSettingsScreen);
}

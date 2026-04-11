import { getNavigator } from '@websoil/engine';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import {
    createAttributeValueInput,
    isBlankAttributeValue,
} from './attribute_value_input';
import ChipsInput from './chips_input';
import type { CreateDraftState, ItemTableColumn, ItemTableViewConfig } from './item_table_types';

export function buildCreateRow(
    config: ItemTableViewConfig,
    draft: CreateDraftState,
    creating: boolean,
    attributeKinds: Record<string, AttributeKind>,
    onSubmit: () => void,
): HTMLTableRowElement {
    const row = document.createElement('tr');
    row.className = 'item-table-create-row';

    for (const column of config.columns) {
        const cell = document.createElement('td');
        cell.appendChild(buildCreateCell(column, draft, attributeKinds, onSubmit));
        row.appendChild(cell);
    }

    const actionCell = document.createElement('td');
    actionCell.className = 'item-table-actions';

    const submit = document.createElement('button');
    submit.type = 'button';
    submit.textContent = creating ? 'Creating...' : (config.createRow?.submitLabel ?? 'Create');
    submit.disabled = creating;
    submit.addEventListener('click', onSubmit);

    actionCell.appendChild(submit);
    row.appendChild(actionCell);
    return row;
}

export function buildCreateCell(
    column: ItemTableColumn,
    draft: CreateDraftState,
    attributeKinds: Record<string, AttributeKind>,
    onSubmit: () => void,
): HTMLElement {
    if (column.kind === 'title') {
        const input = document.createElement('input');
        input.type = 'text';
        input.placeholder = 'New item title';
        input.value = draft.title;
        input.dataset.itemTableCreateTitle = 'true';
        input.addEventListener('input', () => {
            draft.title = input.value;
        });
        input.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') {
                event.preventDefault();
                onSubmit();
            }
        });
        return input;
    }

    if (column.kind === 'types') {
        const chips = new ChipsInput();
        chips.value = draft.types;
        chips.OnClickItem = (name) => { getNavigator().openType(name); };
        chips.addEventListener('chips-add', () => { draft.types = chips.value.slice(); });
        chips.addEventListener('chips-remove', () => { draft.types = chips.value.slice(); });
        return chips;
    }

    const kind = attributeKinds[column.attributeKey];
    const controlOptions: Parameters<typeof createAttributeValueInput>[0] = {
        allowEmpty: true,
        attributeKey: column.attributeKey,
        onValueChange: (value) => {
            if (isBlankAttributeValue(value, kind)) {
                delete draft.attributes[column.attributeKey];
                return;
            }
            draft.attributes[column.attributeKey] = value;
        },
        value: draft.attributes[column.attributeKey],
    };
    if (kind) controlOptions.kind = kind;

    return createAttributeValueInput(controlOptions).element;
}

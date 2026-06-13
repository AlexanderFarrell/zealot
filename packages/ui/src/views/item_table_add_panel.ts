import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { ItemType } from '@zealot/domain/src/item_type';
import { createAttributeValueInput, isBlankAttributeValue } from './attribute_value_input';
import type { CreateDraftState, ItemTableCreateRowConfig } from './item_table_types';

const itemTypeApi = new ItemTypeAPI('/api');

export function buildAddPanel(
    createRowConfig: ItemTableCreateRowConfig,
    draft: CreateDraftState,
    attributeKinds: Record<string, AttributeKind>,
    startOpen: boolean,
    onSubmit: () => void,
    onReset: () => void,
): HTMLElement {
    const panel = document.createElement('div');
    panel.className = 'item-table-add-panel item-table-add-panel--collapsed';

    const label = createRowConfig.submitLabel ?? 'Add';

    const openBtn = document.createElement('button');
    openBtn.type = 'button';
    openBtn.className = 'item-table-add-panel__open';
    openBtn.textContent = `+ ${label}`;
    panel.appendChild(openBtn);

    const form = document.createElement('div');
    form.className = 'item-table-add-panel__form';
    panel.appendChild(form);

    let selectedType: ItemType | null = null;

    const focusTitle = (): void => {
        window.requestAnimationFrame(() => {
            (form.querySelector('input[data-panel-title]') as HTMLInputElement | null)?.focus();
        });
    };

    const expand = (): void => {
        panel.classList.remove('item-table-add-panel--collapsed');
        renderForm();
        focusTitle();
    };

    const collapse = (): void => {
        panel.classList.add('item-table-add-panel--collapsed');
        selectedType = null;
        onReset();
    };

    openBtn.addEventListener('click', expand);

    const renderForm = (): void => {
        form.innerHTML = '';

        // Type row
        const typeField = document.createElement('div');
        typeField.className = 'item-table-add-panel__field';

        const typeLabel = document.createElement('label');
        typeLabel.textContent = 'Type';

        const typeSelect = document.createElement('select');
        const noneOption = document.createElement('option');
        noneOption.value = '';
        noneOption.textContent = '— none —';
        typeSelect.appendChild(noneOption);

        typeField.append(typeLabel, typeSelect);
        form.appendChild(typeField);

        // Populate type options async, then render attribute fields
        void itemTypeApi.get_summaries().then((summaries) => {
            summaries
                .slice()
                .sort((a, b) => a.Name.localeCompare(b.Name))
                .forEach((summary) => {
                    const option = document.createElement('option');
                    option.value = summary.Name;
                    option.textContent = summary.Name;
                    typeSelect.appendChild(option);
                });

            if (selectedType) {
                typeSelect.value = selectedType.Name;
            } else if (createRowConfig.defaultTypes?.length) {
                typeSelect.value = createRowConfig.defaultTypes[0] ?? '';
            }
        });

        typeSelect.addEventListener('change', () => {
            const name = typeSelect.value;
            if (!name) {
                selectedType = null;
                draft.types = [];
                renderAttributeFields(null);
                return;
            }
            void itemTypeApi.get_by_name(name).then((type) => {
                selectedType = type;
                draft.types = type ? [type.Name] : [];
                renderAttributeFields(type);
            });
        });

        // Title row
        const titleField = document.createElement('div');
        titleField.className = 'item-table-add-panel__field';

        const titleLabel = document.createElement('label');
        titleLabel.textContent = 'Title';

        const titleInput = document.createElement('input');
        titleInput.type = 'text';
        titleInput.placeholder = 'New item title';
        titleInput.value = draft.title;
        titleInput.dataset.panelTitle = 'true';
        titleInput.addEventListener('input', () => { draft.title = titleInput.value; });
        titleInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') { e.preventDefault(); onSubmit(); }
            if (e.key === 'Escape') { e.preventDefault(); collapse(); }
        });

        titleField.append(titleLabel, titleInput);
        form.appendChild(titleField);

        // Attribute fields placeholder
        const attrContainer = document.createElement('div');
        attrContainer.className = 'item-table-add-panel__attrs';
        form.appendChild(attrContainer);

        // Actions row
        const actions = document.createElement('div');
        actions.className = 'item-table-add-panel__actions';

        const submitBtn = document.createElement('button');
        submitBtn.type = 'button';
        submitBtn.textContent = label;
        submitBtn.addEventListener('click', onSubmit);

        const cancelBtn = document.createElement('button');
        cancelBtn.type = 'button';
        cancelBtn.className = 'item-table-add-panel__cancel';
        cancelBtn.textContent = 'Cancel';
        cancelBtn.addEventListener('click', collapse);

        actions.append(submitBtn, cancelBtn);
        form.appendChild(actions);

        const renderAttributeFields = (type: ItemType | null): void => {
            attrContainer.innerHTML = '';
            if (!type?.RequiredAttributes.length) return;

            for (const key of type.RequiredAttributes) {
                const alreadyProvided =
                    (key === 'Parent' && createRowConfig.contextItemId != null) ||
                    (createRowConfig.defaultAttributes != null && key in createRowConfig.defaultAttributes);
                if (alreadyProvided) continue;

                const kind = attributeKinds[key];
                const field = document.createElement('div');
                field.className = 'item-table-add-panel__field';

                const lbl = document.createElement('label');
                lbl.textContent = key;

                const opts: Parameters<typeof createAttributeValueInput>[0] = {
                    allowEmpty: true,
                    attributeKey: key,
                    onValueChange: (value) => {
                        if (isBlankAttributeValue(value, kind)) {
                            delete draft.attributes[key];
                        } else {
                            draft.attributes[key] = value;
                        }
                    },
                    value: draft.attributes[key],
                };
                if (kind) opts.kind = kind;
                const binding = createAttributeValueInput(opts);

                binding.element.addEventListener('keydown', (e: KeyboardEvent) => {
                    if (e.key === 'Enter') { e.preventDefault(); onSubmit(); }
                    if (e.key === 'Escape') { e.preventDefault(); collapse(); }
                });

                field.append(lbl, binding.element);
                attrContainer.appendChild(field);
            }
        };

        // Restore attribute fields if a type is already selected
        if (selectedType) {
            renderAttributeFields(selectedType);
        }
    };

    if (startOpen) {
        panel.classList.remove('item-table-add-panel--collapsed');
        // Restore selectedType from draft before rendering form (e.g. after success re-render)
        const restoredTypeName = draft.types[0];
        if (restoredTypeName) {
            void itemTypeApi.get_by_name(restoredTypeName).then((type) => {
                selectedType = type;
                renderForm();
                focusTitle();
            });
        } else {
            renderForm();
            focusTitle();
        }
    }

    return panel;
}

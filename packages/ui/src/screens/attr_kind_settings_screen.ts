import { BaseElementEmpty } from '@websoil/engine';
import { AttributeKindAPI } from '@zealot/api/src/attribute_kind';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import { AttributeBaseTypesArray } from '@zealot/domain/src/attribute';
import type { AddAttributeKindDto, AttributeBaseType, AttributeConfig } from '@zealot/domain/src/attribute';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';

const attributeKindApi = new AttributeKindAPI('/api');

// All scalar base types — valid as list inner types (excludes 'list' itself)
const ScalarBaseTypesArray = AttributeBaseTypesArray.filter((t) => t !== 'list');

export class AttrKindSettingsScreen extends BaseElementEmpty {
    private createError: string | null = null;
    private creating = false;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;

        this.className = 'attr-kind-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'attr-kind-settings-shell';

        const header = document.createElement('div');
        header.className = 'attr-kind-settings-header';

        const heading = document.createElement('h2');
        heading.textContent = 'Attribute Kinds';
        const copy = document.createElement('p');
        copy.className = 'tool-muted';
        copy.textContent = 'Manage the attribute kinds used to describe items.';
        header.append(heading, copy);

        const form = this.buildAddForm();
        shell.append(header, form);

        if (this.createError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = this.createError;
            shell.appendChild(error);
        }

        const content = document.createElement('div');
        content.className = 'attr-kind-settings-content';
        content.appendChild(new LoadingSpinner());
        shell.appendChild(content);

        this.appendChild(shell);

        try {
            const kinds = await attributeKindApi.get_all();
            if (renderId !== this.renderId) {
                return;
            }
            this.renderKinds(content, kinds);
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            content.innerHTML = '';
            const message = document.createElement('p');
            message.className = 'tool-error';
            message.textContent = error instanceof Error && error.message
                ? error.message
                : 'Failed to load attribute kinds.';
            content.appendChild(message);
        }
    }

    private buildAddForm(): HTMLFormElement {
        const form = document.createElement('form');
        form.className = 'attr-kind-settings-create';

        const keyField = document.createElement('label');
        keyField.className = 'tool-field';
        keyField.innerHTML = '<span class="tool-label">Key</span>';
        const keyInput = document.createElement('input');
        keyInput.type = 'text';
        keyInput.name = 'key';
        keyInput.placeholder = 'e.g. priority';
        keyInput.disabled = this.creating;
        keyField.appendChild(keyInput);

        const descField = document.createElement('label');
        descField.className = 'tool-field';
        descField.innerHTML = '<span class="tool-label">Description</span>';
        const descInput = document.createElement('input');
        descInput.type = 'text';
        descInput.name = 'description';
        descInput.placeholder = 'Optional description';
        descInput.disabled = this.creating;
        descField.appendChild(descInput);

        const typeField = document.createElement('label');
        typeField.className = 'tool-field';
        typeField.innerHTML = '<span class="tool-label">Base Type</span>';
        const typeSelect = buildBaseTypeSelect();
        typeSelect.disabled = this.creating;
        typeField.appendChild(typeSelect);

        // Holds current config state while the user fills out the form
        let currentConfig: AttributeConfig = defaultConfigForType(typeSelect.value as AttributeBaseType);

        const configSection = document.createElement('div');
        configSection.className = 'attr-kind-config-section';

        const rebuildConfigSection = () => {
            configSection.innerHTML = '';
            const fields = buildConfigFields(
                typeSelect.value as AttributeBaseType,
                currentConfig,
                (newConfig) => { currentConfig = newConfig; },
            );
            if (fields) {
                configSection.appendChild(fields);
            }
        };

        rebuildConfigSection();

        typeSelect.addEventListener('change', () => {
            currentConfig = defaultConfigForType(typeSelect.value as AttributeBaseType);
            rebuildConfigSection();
        });

        const submit = document.createElement('button');
        submit.type = 'submit';
        submit.disabled = this.creating;
        submit.textContent = this.creating ? 'Adding…' : 'Add Kind';

        form.append(keyField, descField, typeField, configSection, submit);

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            const key = keyInput.value.trim();
            const description = descInput.value.trim();
            const base_type = typeSelect.value as AttributeBaseType;
            if (!key) {
                this.createError = 'Key is required.';
                void this.render();
                return;
            }
            void this.addKind({ key, description, base_type, config: currentConfig });
        });

        return form;
    }

    private renderKinds(container: HTMLElement, kinds: AttributeKind[]): void {
        container.innerHTML = '';

        const systemKinds = kinds.filter((k) => k.IsSystem);
        const userKinds = kinds.filter((k) => !k.IsSystem);

        const systemSection = document.createElement('div');
        systemSection.className = 'attr-kind-section';
        const systemHeading = document.createElement('h3');
        systemHeading.textContent = 'System Kinds';
        const systemNote = document.createElement('p');
        systemNote.className = 'tool-muted';
        systemNote.textContent = 'These are built in and cannot be edited.';
        systemSection.append(systemHeading, systemNote);

        if (systemKinds.length > 0) {
            systemSection.appendChild(buildKindTable(systemKinds, true, () => {}));
        }

        const userSection = document.createElement('div');
        userSection.className = 'attr-kind-section';
        const userHeading = document.createElement('h3');
        userHeading.textContent = 'User Kinds';
        userSection.appendChild(userHeading);

        if (userKinds.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted';
            empty.textContent = 'Attribute kinds you add will appear here.';
            userSection.appendChild(empty);
        } else {
            userSection.appendChild(buildKindTable(userKinds, false, (kind) => {
                void this.deleteKind(kind);
            }));
        }

        container.append(systemSection, userSection);
    }

    private async addKind(dto: AddAttributeKindDto): Promise<void> {
        if (this.creating) {
            return;
        }
        this.creating = true;
        this.createError = null;
        void this.render();

        try {
            await attributeKindApi.add(dto);
        } catch (error) {
            this.createError = await getErrorMessage(error, 'Failed to add attribute kind.');
        }

        this.creating = false;
        void this.render();
    }

    private async deleteKind(kind: AttributeKind): Promise<void> {
        const confirmed = await ConfirmDialog.show(`Delete the "${kind.Key}" attribute kind?`);
        if (!confirmed) {
            return;
        }

        try {
            await attributeKindApi.removeByKey(kind.Key);
            void this.render();
        } catch (error) {
            this.createError = await getErrorMessage(error, 'Failed to delete attribute kind.');
            void this.render();
        }
    }
}

function buildKindTable(
    kinds: AttributeKind[],
    readonly: boolean,
    onDelete: (kind: AttributeKind) => void,
): HTMLTableElement {
    const table = document.createElement('table');
    table.className = 'attr-kind-table';
    table.innerHTML = `
        <thead>
            <tr>
                <th>Key</th>
                <th>Description</th>
                <th>Base Type</th>
                <th>Config</th>
                ${readonly ? '' : '<th></th>'}
            </tr>
        </thead>
    `;

    const tbody = document.createElement('tbody');
    kinds.forEach((kind) => {
        const row = document.createElement('tr');

        if (readonly) {
            const keyCell = document.createElement('td');
            keyCell.textContent = kind.Key;
            const descCell = document.createElement('td');
            descCell.textContent = kind.Description;
            const typeCell = document.createElement('td');
            typeCell.textContent = kind.BaseType;
            const configCell = document.createElement('td');
            configCell.className = 'tool-muted';
            configCell.textContent = describeConfig(kind.BaseType, kind.Config);
            row.append(keyCell, descCell, typeCell, configCell);
        } else {
            const keyCell = document.createElement('td');
            const keyInput = document.createElement('input');
            keyInput.type = 'text';
            keyInput.value = kind.Key;
            keyInput.addEventListener('change', () => {
                kind.Key = keyInput.value;
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, key: kind.Key });
            });
            keyCell.appendChild(keyInput);

            const descCell = document.createElement('td');
            const descInput = document.createElement('input');
            descInput.type = 'text';
            descInput.value = kind.Description;
            descInput.addEventListener('change', () => {
                kind.Description = descInput.value;
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, description: kind.Description });
            });
            descCell.appendChild(descInput);

            const typeCell = document.createElement('td');
            const typeSelect = buildBaseTypeSelect();
            typeSelect.value = kind.BaseType;

            const configCell = document.createElement('td');
            configCell.className = 'config-cell';

            const rebuildConfigCell = () => {
                configCell.innerHTML = '';
                const fields = buildConfigFields(kind.BaseType, kind.Config, (newConfig) => {
                    kind.Config = newConfig;
                    void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, config: kind.Config });
                });
                if (fields) {
                    configCell.appendChild(fields);
                } else {
                    configCell.textContent = '—';
                }
            };

            typeSelect.addEventListener('change', () => {
                kind.BaseType = typeSelect.value as AttributeBaseType;
                kind.Config = defaultConfigForType(kind.BaseType);
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, base_type: kind.BaseType, config: kind.Config });
                rebuildConfigCell();
            });
            typeCell.appendChild(typeSelect);

            rebuildConfigCell();

            const actionsCell = document.createElement('td');
            const deleteButton = document.createElement('button');
            deleteButton.type = 'button';
            deleteButton.textContent = 'Delete';
            deleteButton.addEventListener('click', () => onDelete(kind));
            actionsCell.appendChild(deleteButton);

            row.append(keyCell, descCell, typeCell, configCell, actionsCell);
        }

        tbody.appendChild(row);
    });

    table.appendChild(tbody);
    return table;
}

function buildBaseTypeSelect(): HTMLSelectElement {
    const select = document.createElement('select');
    select.name = 'base_type';
    AttributeBaseTypesArray.forEach((type) => {
        const option = document.createElement('option');
        option.value = type;
        option.textContent = type.charAt(0).toUpperCase() + type.slice(1);
        select.appendChild(option);
    });
    return select;
}

function buildScalarTypeSelect(currentValue: string): HTMLSelectElement {
    const select = document.createElement('select');
    ScalarBaseTypesArray.forEach((type) => {
        const option = document.createElement('option');
        option.value = type;
        option.textContent = type.charAt(0).toUpperCase() + type.slice(1);
        select.appendChild(option);
    });
    select.value = currentValue || 'text';
    return select;
}

function buildConfigFields(
    baseType: AttributeBaseType,
    config: AttributeConfig,
    onChange: (newConfig: AttributeConfig) => void,
): HTMLElement | null {
    switch (baseType) {
        case 'list':
            return buildListConfigFields(config, onChange);
        case 'dropdown':
            return buildDropdownConfigFields(config, onChange);
        case 'integer':
        case 'decimal':
            return buildNumericConfigFields(config, onChange);
        case 'text':
            return buildTextConfigFields(config, onChange);
        default:
            return null;
    }
}

function buildListConfigFields(
    config: AttributeConfig,
    onChange: (newConfig: AttributeConfig) => void,
): HTMLElement {
    const container = document.createElement('div');
    container.className = 'config-fields';

    const label = document.createElement('label');
    label.className = 'config-field';
    const span = document.createElement('span');
    span.textContent = 'Inner Type';
    const select = buildScalarTypeSelect(config.list_type ?? 'text');
    select.addEventListener('change', () => {
        onChange({ ...config, list_type: select.value as AttributeBaseType });
    });
    label.append(span, select);
    container.appendChild(label);
    return container;
}

function buildDropdownConfigFields(
    config: AttributeConfig,
    onChange: (newConfig: AttributeConfig) => void,
): HTMLElement {
    const container = document.createElement('div');
    container.className = 'config-fields';

    const label = document.createElement('label');
    label.className = 'config-field';
    const span = document.createElement('span');
    span.textContent = 'Values (comma-separated)';
    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = 'e.g. todo, working, complete';
    input.value = (config.values ?? []).join(', ');
    input.addEventListener('change', () => {
        const values = input.value.split(',').map((v) => v.trim()).filter((v) => v.length > 0);
        onChange({ ...config, values });
    });
    label.append(span, input);
    container.appendChild(label);
    return container;
}

function buildNumericConfigFields(
    config: AttributeConfig,
    onChange: (newConfig: AttributeConfig) => void,
): HTMLElement {
    const container = document.createElement('div');
    container.className = 'config-fields';

    const minLabel = document.createElement('label');
    minLabel.className = 'config-field';
    const minSpan = document.createElement('span');
    minSpan.textContent = 'Min';
    const minInput = document.createElement('input');
    minInput.type = 'number';
    minInput.placeholder = 'None';
    if (config.min !== undefined && config.min !== null) minInput.value = String(config.min);
    minLabel.append(minSpan, minInput);

    const maxLabel = document.createElement('label');
    maxLabel.className = 'config-field';
    const maxSpan = document.createElement('span');
    maxSpan.textContent = 'Max';
    const maxInput = document.createElement('input');
    maxInput.type = 'number';
    maxInput.placeholder = 'None';
    if (config.max !== undefined && config.max !== null) maxInput.value = String(config.max);
    maxLabel.append(maxSpan, maxInput);

    const save = () => {
        const newConfig: AttributeConfig = { ...config };
        if (minInput.value.trim() !== '') newConfig.min = Number(minInput.value);
        else delete newConfig.min;
        if (maxInput.value.trim() !== '') newConfig.max = Number(maxInput.value);
        else delete newConfig.max;
        onChange(newConfig);
    };
    minInput.addEventListener('change', save);
    maxInput.addEventListener('change', save);

    container.append(minLabel, maxLabel);
    return container;
}

function buildTextConfigFields(
    config: AttributeConfig,
    onChange: (newConfig: AttributeConfig) => void,
): HTMLElement {
    const container = document.createElement('div');
    container.className = 'config-fields';

    const minLenLabel = document.createElement('label');
    minLenLabel.className = 'config-field';
    const minLenSpan = document.createElement('span');
    minLenSpan.textContent = 'Min Length';
    const minLenInput = document.createElement('input');
    minLenInput.type = 'number';
    minLenInput.min = '0';
    minLenInput.placeholder = 'None';
    if (config.min_len !== undefined && config.min_len !== null) minLenInput.value = String(config.min_len);
    minLenLabel.append(minLenSpan, minLenInput);

    const maxLenLabel = document.createElement('label');
    maxLenLabel.className = 'config-field';
    const maxLenSpan = document.createElement('span');
    maxLenSpan.textContent = 'Max Length';
    const maxLenInput = document.createElement('input');
    maxLenInput.type = 'number';
    maxLenInput.min = '0';
    maxLenInput.placeholder = 'None';
    if (config.max_len !== undefined && config.max_len !== null) maxLenInput.value = String(config.max_len);
    maxLenLabel.append(maxLenSpan, maxLenInput);

    const patternLabel = document.createElement('label');
    patternLabel.className = 'config-field';
    const patternSpan = document.createElement('span');
    patternSpan.textContent = 'Pattern (regex)';
    const patternInput = document.createElement('input');
    patternInput.type = 'text';
    patternInput.placeholder = 'None';
    if (config.pattern) patternInput.value = config.pattern;
    patternLabel.append(patternSpan, patternInput);

    const save = () => {
        const newConfig: AttributeConfig = { ...config };
        if (minLenInput.value.trim() !== '') newConfig.min_len = Number(minLenInput.value);
        else delete newConfig.min_len;
        if (maxLenInput.value.trim() !== '') newConfig.max_len = Number(maxLenInput.value);
        else delete newConfig.max_len;
        if (patternInput.value.trim() !== '') newConfig.pattern = patternInput.value.trim();
        else delete newConfig.pattern;
        onChange(newConfig);
    };
    minLenInput.addEventListener('change', save);
    maxLenInput.addEventListener('change', save);
    patternInput.addEventListener('change', save);

    container.append(minLenLabel, maxLenLabel, patternLabel);
    return container;
}

function describeConfig(baseType: AttributeBaseType, config: AttributeConfig): string {
    switch (baseType) {
        case 'list':
            return `inner: ${config.list_type ?? 'text'}`;
        case 'dropdown':
            return config.values?.join(', ') ?? '—';
        case 'integer':
        case 'decimal': {
            const parts: string[] = [];
            if (config.min !== undefined) parts.push(`min: ${config.min}`);
            if (config.max !== undefined) parts.push(`max: ${config.max}`);
            return parts.join(', ') || '—';
        }
        case 'text': {
            const parts: string[] = [];
            if (config.min_len !== undefined) parts.push(`min_len: ${config.min_len}`);
            if (config.max_len !== undefined) parts.push(`max_len: ${config.max_len}`);
            if (config.pattern) parts.push(`pattern: ${config.pattern}`);
            return parts.join(', ') || '—';
        }
        default:
            return '—';
    }
}

function defaultConfigForType(base_type: AttributeBaseType): AttributeConfig {
    switch (base_type) {
        case 'dropdown':
            return { values: ['todo', 'working', 'complete'] };
        case 'list':
            return { list_type: 'text' };
        default:
            return {};
    }
}

async function getErrorMessage(error: unknown, fallback: string): Promise<string> {
    const response = (error as Error & { response?: Response }).response;
    if (response) {
        try {
            const text = await response.text();
            if (text.trim() !== '') {
                return text;
            }
        } catch {
            // Ignore response parsing errors and fall back.
        }
    }
    return error instanceof Error && error.message ? error.message : fallback;
}

if (!customElements.get('attr-kind-settings-screen')) {
    customElements.define('attr-kind-settings-screen', AttrKindSettingsScreen);
}

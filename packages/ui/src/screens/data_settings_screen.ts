import { BaseElementEmpty, Popups, get_json } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import type { AddItemDto } from '@zealot/domain/src/item';
import {
    createAttributeValueInput,
    loadAttributeKinds,
    type AttributeValueInputBinding,
} from '../views/attribute_value_input';
import { ItemChipsInput } from '../views/item_chips_input';
import type { AttributeKind } from '@zealot/domain/src/attribute';

const itemApi = new ItemAPI('/api');

interface ParsedItem {
    title: string;
    content: string;
    attributes: Record<string, unknown>;
    types: string[];
    fileName: string;
}

interface ImportRow {
    parsed: ParsedItem;
    status: 'pending' | 'importing' | 'done' | 'error';
    error: string | null;
    statusEl: HTMLElement;
}

// Extra attribute row the user adds in the metadata panel
interface MetaAttrRow {
    key: string;
    binding: AttributeValueInputBinding;
    el: HTMLElement;
}

function parseSimpleYamlValue(raw: string): unknown {
    const trimmed = raw.trim();
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        return trimmed
            .slice(1, -1)
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean);
    }
    if (trimmed === 'true') return true;
    if (trimmed === 'false') return false;
    const num = Number(trimmed);
    if (!isNaN(num) && trimmed !== '') return num;
    return trimmed;
}

async function parseMarkdownFile(file: File): Promise<ParsedItem> {
    const text = await file.text();
    let title = file.name.replace(/\.md$/i, '');
    let content = text;
    const attributes: Record<string, unknown> = {};
    const types: string[] = [];

    if (text.startsWith('---')) {
        const end = text.indexOf('\n---', 3);
        if (end !== -1) {
            const frontmatter = text.slice(3, end).trim();
            content = text.slice(end + 4).trimStart();

            for (const line of frontmatter.split('\n')) {
                const colon = line.indexOf(':');
                if (colon === -1) continue;
                const key = line.slice(0, colon).trim();
                const val = parseSimpleYamlValue(line.slice(colon + 1));

                if (key === 'title' && typeof val === 'string') {
                    title = val;
                } else if (key === 'types' || key === 'tags') {
                    if (Array.isArray(val)) {
                        types.push(...val.map(String));
                    } else if (typeof val === 'string' && val) {
                        types.push(val);
                    }
                } else {
                    attributes[key] = val;
                }
            }
        }
    }

    return { title, content, attributes, types, fileName: file.name };
}

export class DataSettingsScreen extends BaseElementEmpty {
    private rows: ImportRow[] = [];
    private importing = false;
    private wizardEl: HTMLElement | null = null;
    private importAllBtn: HTMLButtonElement | null = null;
    private summaryEl: HTMLElement | null = null;

    // Metadata panel state
    private metaParentChips: ItemChipsInput | null = null;
    private metaTypesInput: HTMLInputElement | null = null;
    private metaAttrRows: MetaAttrRow[] = [];
    private metaAttrListEl: HTMLElement | null = null;
    private attributeKinds: Record<string, AttributeKind> = {};

    render() {
        this.className = 'data-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'data-settings-shell';

        const heading = document.createElement('h2');
        heading.textContent = 'Data Settings';
        shell.appendChild(heading);

        shell.appendChild(this.buildImportSection());
        shell.appendChild(this.buildExportSection());

        this.appendChild(shell);

        // Load attribute kinds in background for the metadata panel
        void loadAttributeKinds().then((kinds) => { this.attributeKinds = kinds; });
    }

    private buildImportSection(): HTMLElement {
        const section = document.createElement('section');
        section.className = 'data-settings-section';

        const h3 = document.createElement('h3');
        h3.textContent = 'Import';
        section.appendChild(h3);

        const desc = document.createElement('p');
        desc.className = 'tool-muted';
        desc.textContent = 'Import Markdown files as Zealot items. Frontmatter (title, types, and attributes) is parsed automatically.';
        section.appendChild(desc);

        const fileRow = document.createElement('div');
        fileRow.className = 'data-settings-file-row';

        const fileInput = document.createElement('input');
        fileInput.type = 'file';
        fileInput.accept = '.md';
        fileInput.multiple = true;
        fileInput.style.display = 'none';

        const selectBtn = document.createElement('button');
        selectBtn.type = 'button';
        selectBtn.textContent = 'Select Markdown Files…';
        selectBtn.addEventListener('click', () => fileInput.click());

        fileInput.addEventListener('change', () => {
            const files = Array.from(fileInput.files ?? []);
            if (files.length === 0) return;
            void this.onFilesSelected(files);
            fileInput.value = '';
        });

        fileRow.append(fileInput, selectBtn);
        section.appendChild(fileRow);

        const wizard = document.createElement('div');
        wizard.className = 'data-import-wizard';
        wizard.hidden = true;
        this.wizardEl = wizard;
        section.appendChild(wizard);

        return section;
    }

    private async onFilesSelected(files: File[]): Promise<void> {
        this.rows = [];
        this.importing = false;
        this.metaAttrRows = [];

        const wizard = this.wizardEl;
        if (!wizard) return;

        wizard.innerHTML = '';
        wizard.hidden = false;

        const parsed = await Promise.all(files.map(parseMarkdownFile));

        // ── Metadata panel ────────────────────────────────────────────────────
        wizard.appendChild(this.buildMetadataPanel());

        // ── Preview table ─────────────────────────────────────────────────────
        const tableHeading = document.createElement('h4');
        tableHeading.className = 'data-import-table-heading';
        tableHeading.textContent = `${parsed.length} file${parsed.length !== 1 ? 's' : ''} selected`;
        wizard.appendChild(tableHeading);

        const table = document.createElement('table');
        table.className = 'data-import-table';
        table.innerHTML = `<thead><tr>
            <th>File</th>
            <th>Title</th>
            <th>Types</th>
            <th>Content preview</th>
            <th>Status</th>
        </tr></thead>`;

        const tbody = document.createElement('tbody');

        for (const p of parsed) {
            const statusCell = document.createElement('td');
            statusCell.textContent = 'Pending';
            statusCell.className = 'import-status-pending';

            const row: ImportRow = {
                parsed: p,
                status: 'pending',
                error: null,
                statusEl: statusCell,
            };

            const tr = document.createElement('tr');
            const fileCell = document.createElement('td');
            fileCell.textContent = p.fileName;

            const titleCell = document.createElement('td');
            titleCell.textContent = p.title;

            const typesCell = document.createElement('td');
            typesCell.textContent = p.types.join(', ') || '—';

            const previewCell = document.createElement('td');
            previewCell.className = 'import-preview';
            previewCell.textContent = p.content.slice(0, 80) + (p.content.length > 80 ? '…' : '');

            tr.append(fileCell, titleCell, typesCell, previewCell, statusCell);
            tbody.appendChild(tr);
            this.rows.push(row);
        }

        table.appendChild(tbody);
        wizard.appendChild(table);

        // ── Actions ───────────────────────────────────────────────────────────
        const actions = document.createElement('div');
        actions.className = 'data-import-actions';

        const importAllBtn = document.createElement('button');
        importAllBtn.type = 'button';
        importAllBtn.textContent = `Import ${parsed.length} item${parsed.length !== 1 ? 's' : ''}`;
        importAllBtn.addEventListener('click', () => void this.runImport());
        this.importAllBtn = importAllBtn;

        const cancelBtn = document.createElement('button');
        cancelBtn.type = 'button';
        cancelBtn.textContent = 'Cancel';
        cancelBtn.addEventListener('click', () => {
            wizard.hidden = true;
            wizard.innerHTML = '';
            this.rows = [];
            this.metaAttrRows = [];
        });

        const summary = document.createElement('div');
        summary.className = 'data-import-summary';
        summary.hidden = true;
        this.summaryEl = summary;

        actions.append(importAllBtn, cancelBtn);
        wizard.append(actions, summary);
    }

    // ── Metadata panel ────────────────────────────────────────────────────────

    private buildMetadataPanel(): HTMLElement {
        const panel = document.createElement('div');
        panel.className = 'data-import-meta-panel';

        const heading = document.createElement('h4');
        heading.textContent = 'Apply to all items';
        panel.appendChild(heading);

        const note = document.createElement('p');
        note.className = 'tool-muted';
        note.textContent = 'These values are added to every imported item. Per-file frontmatter takes precedence for the same key.';
        panel.appendChild(note);

        // Parent attribute (item ID stored as the "Parent" attribute)
        const parentLabel = document.createElement('label');
        parentLabel.className = 'tool-field';
        const parentLabelText = document.createElement('span');
        parentLabelText.className = 'tool-label';
        parentLabelText.textContent = 'Parent';
        parentLabel.appendChild(parentLabelText);

        const parentChips = new ItemChipsInput();
        parentLabel.appendChild(parentChips);
        this.metaParentChips = parentChips;
        panel.appendChild(parentLabel);

        // Types field
        const typesLabel = document.createElement('label');
        typesLabel.className = 'tool-field';
        const typesLabelText = document.createElement('span');
        typesLabelText.className = 'tool-label';
        typesLabelText.textContent = 'Types (comma-separated)';
        typesLabel.appendChild(typesLabelText);

        const typesInput = document.createElement('input');
        typesInput.type = 'text';
        typesInput.placeholder = 'e.g. Note, Blog';
        typesLabel.appendChild(typesInput);
        this.metaTypesInput = typesInput;
        panel.appendChild(typesLabel);

        // Extra attributes list
        const attrSection = document.createElement('div');
        attrSection.className = 'data-import-meta-attrs';

        const attrHeading = document.createElement('span');
        attrHeading.className = 'tool-label';
        attrHeading.textContent = 'Additional attributes';
        attrSection.appendChild(attrHeading);

        const attrList = document.createElement('div');
        attrList.className = 'data-import-meta-attr-list';
        this.metaAttrListEl = attrList;
        attrSection.appendChild(attrList);

        attrSection.appendChild(this.buildMetaAttrAddRow());
        panel.appendChild(attrSection);

        return panel;
    }

    private buildMetaAttrAddRow(): HTMLElement {
        const row = document.createElement('div');
        row.className = 'attribute data-import-meta-attr-add';

        const keyInput = document.createElement('input');
        keyInput.type = 'text';
        keyInput.placeholder = 'Key';

        const valueSpan = document.createElement('span');
        valueSpan.setAttribute('name', 'value_view');

        let currentBinding: AttributeValueInputBinding = createAttributeValueInput({
            attributeKey: '',
            value: null,
        });
        valueSpan.appendChild(currentBinding.element);

        keyInput.addEventListener('input', () => {
            const k = keyInput.value.trim();
            const kind = this.attributeKinds[k];
            currentBinding = createAttributeValueInput({
                attributeKey: k,
                value: null,
                ...(kind ? { kind } : {}),
            });
            valueSpan.innerHTML = '';
            valueSpan.appendChild(currentBinding.element);
        });

        const addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.textContent = '+';
        addBtn.title = 'Add attribute';

        const doAdd = () => {
            const k = keyInput.value.trim();
            if (!k) return;
            const v = currentBinding.getValue();

            this.addMetaAttrRow(k, v);

            keyInput.value = '';
            currentBinding = createAttributeValueInput({ attributeKey: '', value: null });
            valueSpan.innerHTML = '';
            valueSpan.appendChild(currentBinding.element);
            keyInput.focus();
        };

        addBtn.addEventListener('click', doAdd);
        keyInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') { e.preventDefault(); doAdd(); }
        });

        row.append(keyInput, valueSpan, addBtn);
        return row;
    }

    private addMetaAttrRow(key: string, value: unknown): void {
        const listEl = this.metaAttrListEl;
        if (!listEl) return;

        const kind = this.attributeKinds[key];
        const binding = createAttributeValueInput({
            attributeKey: key,
            value,
            ...(kind ? { kind } : {}),
        });

        const rowEl = document.createElement('div');
        rowEl.className = 'attribute';

        const keyEl = document.createElement('span');
        keyEl.className = 'attribute-key-label';
        keyEl.textContent = key;

        const valueSpan = document.createElement('span');
        valueSpan.setAttribute('name', 'value_view');
        valueSpan.appendChild(binding.element);

        const delBtn = document.createElement('button');
        delBtn.type = 'button';
        delBtn.textContent = '×';
        delBtn.title = 'Remove';
        delBtn.addEventListener('click', () => {
            this.metaAttrRows = this.metaAttrRows.filter((r) => r.el !== rowEl);
            rowEl.remove();
        });

        rowEl.append(keyEl, valueSpan, delBtn);
        listEl.appendChild(rowEl);

        this.metaAttrRows.push({ key, binding, el: rowEl });
    }

    // ── Import runner ─────────────────────────────────────────────────────────

    private async runImport(): Promise<void> {
        if (this.importing) return;
        this.importing = true;

        if (this.importAllBtn) {
            this.importAllBtn.disabled = true;
            this.importAllBtn.textContent = 'Importing…';
        }

        // Collect metadata panel values
        const metaParentIds = this.metaParentChips?.value ?? [];

        const metaTypes = (this.metaTypesInput?.value ?? '')
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean);

        const metaAttrs: Record<string, unknown> = {};
        if (metaParentIds.length > 0) {
            metaAttrs['Parent'] = metaParentIds;
        }
        for (const { key, binding } of this.metaAttrRows) {
            const v = binding.getValue();
            if (v != null && v !== '') metaAttrs[key] = v;
        }

        let successCount = 0;
        const errors: string[] = [];

        for (const row of this.rows) {
            if (row.status !== 'pending') continue;

            row.status = 'importing';
            row.statusEl.textContent = 'Importing…';
            row.statusEl.className = 'import-status-importing';

            // Merge: meta values are base, per-file values override
            const mergedAttrs: Record<string, unknown> = {
                ...metaAttrs,
                ...row.parsed.attributes,
            };

            const mergedTypes = [
                ...metaTypes,
                ...row.parsed.types.filter((t) => !metaTypes.includes(t)),
            ];

            const dto: AddItemDto = {
                title: row.parsed.title,
                content: row.parsed.content,
            };

            if (Object.keys(mergedAttrs).length > 0) {
                dto.attributes = mergedAttrs;
            }

            if (mergedTypes.length > 0) {
                dto.types = mergedTypes;
            }

            try {
                await itemApi.Add(dto);
                row.status = 'done';
                row.statusEl.textContent = 'Imported';
                row.statusEl.className = 'import-status-done';
                successCount++;
            } catch (e) {
                const msg = (e as Error).message ?? 'Unknown error';
                row.status = 'error';
                row.error = msg;
                row.statusEl.textContent = `Error: ${msg}`;
                row.statusEl.className = 'import-status-error';
                errors.push(`${row.parsed.fileName}: ${msg}`);
            }
        }

        this.importing = false;
        if (this.importAllBtn) {
            this.importAllBtn.disabled = true;
            this.importAllBtn.textContent = 'Done';
        }

        if (this.summaryEl) {
            this.summaryEl.hidden = false;
            if (errors.length === 0) {
                this.summaryEl.textContent = `${successCount} item${successCount !== 1 ? 's' : ''} imported successfully.`;
                this.summaryEl.className = 'data-import-summary data-import-summary--success';
            } else {
                this.summaryEl.textContent = `${successCount} imported, ${errors.length} failed.`;
                this.summaryEl.className = 'data-import-summary data-import-summary--partial';
            }
        }
    }

    // ── Export ────────────────────────────────────────────────────────────────

    private buildExportSection(): HTMLElement {
        const section = document.createElement('section');
        section.className = 'data-settings-section';

        const h3 = document.createElement('h3');
        h3.textContent = 'Export';
        section.appendChild(h3);

        const desc = document.createElement('p');
        desc.className = 'tool-muted';
        desc.textContent = 'Download all items as a JSON backup.';
        section.appendChild(desc);

        const exportBtn = document.createElement('button');
        exportBtn.type = 'button';
        exportBtn.textContent = 'Export All Items as JSON';
        exportBtn.addEventListener('click', () => void this.onExportJson());
        section.appendChild(exportBtn);

        return section;
    }

    private async onExportJson(): Promise<void> {
        try {
            const data = await get_json('/api/item');
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `zealot-items-${new Date().toISOString().slice(0, 10)}.json`;
            a.click();
            URL.revokeObjectURL(url);
        } catch (e) {
            Popups.add_error((e as Error).message ?? 'Export failed.');
        }
    }
}

if (!customElements.get('data-settings-screen')) {
    customElements.define('data-settings-screen', DataSettingsScreen);
}

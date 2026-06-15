import { BaseElementEmpty, Popups } from '@websoil/engine';
import { RuleAPI } from '@zealot/api/src/rule';
import { TRIGGER_KIND_LABELS } from '@zealot/domain/src/rule';
import type { AddRuleDto, Rule, RuleRunResult, TriggerKind, UpdateRuleDto } from '@zealot/domain/src/rule';
import { LoadingSpinner } from '../common/loading_spinner';
import { createScreenHeader } from './screen_header';

const ruleApi = new RuleAPI('/api');

function relativeTime(iso: string | null): string {
    if (!iso) return 'Never';
    const diff = Date.now() - new Date(iso).getTime();
    if (diff < 60_000) return 'just now';
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} min ago`;
    if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} hr ago`;
    return `${Math.floor(diff / 86_400_000)} d ago`;
}

export class RulesScreen extends BaseElementEmpty {
    private view: 'list' | 'edit' = 'list';
    private editingRule: Rule | null = null;
    private renderId = 0;

    async render() {
        this.className = 'rules-screen';
        this.innerHTML = '';
        if (this.view === 'edit') {
            this.renderEditForm();
        } else {
            await this.renderList();
        }
    }

    private async renderList(): Promise<void> {
        const renderId = ++this.renderId;
        const shell = document.createElement('div');
        shell.className = 'rules-screen-shell';

        shell.appendChild(createScreenHeader('Rules', [{
            label: 'New Rule',
            onClick: () => {
                this.editingRule = null;
                this.view = 'edit';
                void this.render();
            },
        }]));

        const loading = new LoadingSpinner();
        shell.appendChild(loading);
        this.appendChild(shell);

        let rules: Rule[];
        try {
            rules = await ruleApi.GetAll();
        } catch (err) {
            if (renderId !== this.renderId) return;
            loading.remove();
            const p = document.createElement('p');
            p.className = 'tool-error';
            p.textContent = err instanceof Error ? err.message : 'Failed to load rules.';
            shell.appendChild(p);
            return;
        }

        if (renderId !== this.renderId) return;
        loading.remove();

        if (rules.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted';
            empty.textContent = 'No rules yet. Create one to get started.';
            shell.appendChild(empty);
            return;
        }

        const table = document.createElement('table');
        table.className = 'rules-list';
        table.innerHTML = `
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Trigger</th>
                    <th>Enabled</th>
                    <th>Last Run</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
        `;

        const tbody = document.createElement('tbody');
        for (const rule of rules) {
            tbody.appendChild(this.buildRuleRow(rule));
        }
        table.appendChild(tbody);
        shell.appendChild(table);
    }

    private buildRuleRow(rule: Rule): HTMLTableRowElement {
        const tr = document.createElement('tr');

        const nameTd = document.createElement('td');
        const nameBtn = document.createElement('button');
        nameBtn.type = 'button';
        nameBtn.className = 'type-row-link';
        nameBtn.textContent = rule.name;
        nameBtn.addEventListener('click', () => {
            this.editingRule = rule;
            this.view = 'edit';
            void this.render();
        });
        nameTd.appendChild(nameBtn);

        const triggerTd = document.createElement('td');
        triggerTd.textContent = TRIGGER_KIND_LABELS[rule.trigger.kind];

        const enabledTd = document.createElement('td');
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = rule.enabled;
        checkbox.addEventListener('change', () => {
            void this.toggleEnabled(rule, checkbox);
        });
        enabledTd.appendChild(checkbox);

        const lastRunTd = document.createElement('td');
        lastRunTd.textContent = relativeTime(rule.last_run_at);

        const statusTd = document.createElement('td');
        const dot = document.createElement('span');
        dot.className = 'rule-status-dot';
        if (rule.last_run_at === null) {
            dot.classList.add('rule-status-never');
            dot.title = 'Never run';
        } else if (rule.last_error) {
            dot.classList.add('rule-status-error');
            dot.title = rule.last_error;
        } else {
            dot.classList.add('rule-status-ok');
            dot.title = 'Last run succeeded';
        }
        statusTd.appendChild(dot);

        const actionsTd = document.createElement('td');
        const editBtn = document.createElement('button');
        editBtn.type = 'button';
        editBtn.textContent = 'Edit';
        editBtn.addEventListener('click', () => {
            this.editingRule = rule;
            this.view = 'edit';
            void this.render();
        });

        const runBtn = document.createElement('button');
        runBtn.type = 'button';
        runBtn.textContent = 'Run Now';
        runBtn.addEventListener('click', () => {
            void this.runRuleFromList(rule, runBtn);
        });

        const deleteBtn = document.createElement('button');
        deleteBtn.type = 'button';
        deleteBtn.textContent = 'Delete';
        deleteBtn.addEventListener('click', () => {
            void this.deleteRule(rule);
        });

        actionsTd.style.display = 'flex';
        actionsTd.style.gap = '6px';
        actionsTd.append(editBtn, runBtn, deleteBtn);

        tr.append(nameTd, triggerTd, enabledTd, lastRunTd, statusTd, actionsTd);
        return tr;
    }

    private async toggleEnabled(rule: Rule, checkbox: HTMLInputElement): Promise<void> {
        try {
            await ruleApi.Update(rule.rule_id, { enabled: checkbox.checked });
        } catch (err) {
            checkbox.checked = !checkbox.checked;
            Popups.add_error(err instanceof Error ? err.message : 'Failed to update rule.');
        }
    }

    private async runRuleFromList(rule: Rule, btn: HTMLButtonElement): Promise<void> {
        btn.disabled = true;
        btn.textContent = 'Running…';
        try {
            const result = await ruleApi.Run(rule.rule_id);
            if (result.success) {
                Popups.add_error(`✓ ${result.output ?? 'Ran successfully'} (${result.duration_ms}ms)`);
            } else {
                Popups.add_error(result.error ?? 'Rule failed with no error message.');
            }
        } catch (err) {
            Popups.add_error(err instanceof Error ? err.message : 'Failed to run rule.');
        } finally {
            btn.disabled = false;
            btn.textContent = 'Run Now';
        }
    }

    private async deleteRule(rule: Rule): Promise<void> {
        if (!confirm(`Delete rule "${rule.name}"?`)) return;
        try {
            await ruleApi.Delete(rule.rule_id);
            void this.render();
        } catch (err) {
            Popups.add_error(err instanceof Error ? err.message : 'Failed to delete rule.');
        }
    }

    private renderEditForm(): void {
        const shell = document.createElement('div');
        shell.className = 'rules-screen-shell';

        shell.appendChild(createScreenHeader(this.editingRule ? `Edit Rule: ${this.editingRule.name}` : 'New Rule'));

        const form = document.createElement('form');
        form.className = 'rule-edit-form';
        form.addEventListener('submit', (e) => {
            e.preventDefault();
            void this.saveRule(form, resultContainer);
        });

        // Name
        const nameLabel = document.createElement('label');
        nameLabel.textContent = 'Name';
        const nameInput = document.createElement('input');
        nameInput.type = 'text';
        nameInput.name = 'name';
        nameInput.required = true;
        nameInput.value = this.editingRule?.name ?? '';
        nameLabel.appendChild(nameInput);
        form.appendChild(nameLabel);

        // Description
        const descLabel = document.createElement('label');
        descLabel.textContent = 'Description';
        const descInput = document.createElement('textarea');
        descInput.name = 'description';
        descInput.rows = 2;
        descInput.value = this.editingRule?.description ?? '';
        descLabel.appendChild(descInput);
        form.appendChild(descLabel);

        // Trigger select
        const triggerLabel = document.createElement('label');
        triggerLabel.textContent = 'Trigger';
        const triggerSelect = document.createElement('select');
        triggerSelect.name = 'trigger';
        for (const [value, label] of Object.entries(TRIGGER_KIND_LABELS)) {
            const opt = document.createElement('option');
            opt.value = value;
            opt.textContent = label;
            if (this.editingRule?.trigger.kind === value) opt.selected = true;
            triggerSelect.appendChild(opt);
        }
        triggerLabel.appendChild(triggerSelect);
        form.appendChild(triggerLabel);

        // Trigger config container
        const configContainer = document.createElement('div');
        configContainer.className = 'rule-trigger-config';
        form.appendChild(configContainer);

        const renderTriggerConfig = (kind: string) => {
            configContainer.innerHTML = '';
            if (kind === 'cron') {
                const lbl = document.createElement('label');
                lbl.textContent = 'Cron Expression';
                const inp = document.createElement('input');
                inp.type = 'text';
                inp.name = 'cron_expression';
                inp.placeholder = '0 9 * * 1';
                if (this.editingRule?.trigger.kind === 'cron') {
                    inp.value = this.editingRule.trigger.expression;
                }
                const hint = document.createElement('small');
                hint.className = 'tool-muted';
                hint.textContent = 'min hour day month weekday';
                lbl.appendChild(inp);
                configContainer.append(lbl, hint);
            } else if (kind === 'interval') {
                const lbl = document.createElement('label');
                lbl.textContent = 'Interval (seconds)';
                const inp = document.createElement('input');
                inp.type = 'number';
                inp.name = 'interval_seconds';
                inp.min = '1';
                if (this.editingRule?.trigger.kind === 'interval') {
                    inp.value = String(this.editingRule.trigger.seconds);
                }
                lbl.appendChild(inp);
                configContainer.appendChild(lbl);
            } else if (kind === 'on_type_assign' || kind === 'on_type_unassign') {
                const lbl = document.createElement('label');
                lbl.textContent = 'Type Name (blank = any)';
                const inp = document.createElement('input');
                inp.type = 'text';
                inp.name = 'type_name';
                if (this.editingRule?.trigger.kind === kind) {
                    inp.value = this.editingRule.trigger.type_name ?? '';
                }
                lbl.appendChild(inp);
                configContainer.appendChild(lbl);
            } else if (kind === 'on_attribute_set') {
                const lbl = document.createElement('label');
                lbl.textContent = 'Attribute Key (blank = any)';
                const inp = document.createElement('input');
                inp.type = 'text';
                inp.name = 'attribute_key';
                if (this.editingRule?.trigger.kind === 'on_attribute_set') {
                    inp.value = this.editingRule.trigger.attribute_key ?? '';
                }
                lbl.appendChild(inp);
                configContainer.appendChild(lbl);
            }
        };

        renderTriggerConfig(triggerSelect.value);
        triggerSelect.addEventListener('change', () => renderTriggerConfig(triggerSelect.value));

        // Enabled
        const enabledLabel = document.createElement('label');
        const enabledRow = document.createElement('div');
        enabledRow.style.display = 'flex';
        enabledRow.style.alignItems = 'center';
        enabledRow.style.gap = '8px';
        const enabledCheck = document.createElement('input');
        enabledCheck.type = 'checkbox';
        enabledCheck.name = 'enabled';
        enabledCheck.checked = this.editingRule?.enabled ?? true;
        const enabledText = document.createElement('span');
        enabledText.textContent = 'Enabled';
        enabledRow.append(enabledCheck, enabledText);
        enabledLabel.appendChild(enabledRow);
        form.appendChild(enabledLabel);

        // Script
        const scriptLabel = document.createElement('label');
        scriptLabel.textContent = 'Lua Script';
        const scriptArea = document.createElement('textarea');
        scriptArea.name = 'script';
        scriptArea.className = 'rule-script-area';
        scriptArea.value = this.editingRule?.script ?? '';
        scriptLabel.appendChild(scriptArea);
        form.appendChild(scriptLabel);

        // Run result container (populated after run)
        const resultContainer = document.createElement('div');

        // Actions
        const actions = document.createElement('div');
        actions.className = 'rule-edit-actions';

        const saveBtn = document.createElement('button');
        saveBtn.type = 'submit';
        saveBtn.textContent = 'Save';

        const cancelBtn = document.createElement('button');
        cancelBtn.type = 'button';
        cancelBtn.textContent = 'Cancel';
        cancelBtn.addEventListener('click', () => {
            this.view = 'list';
            void this.render();
        });

        actions.append(saveBtn, cancelBtn);

        if (this.editingRule) {
            const runBtn = document.createElement('button');
            runBtn.type = 'button';
            runBtn.textContent = 'Run Now';
            runBtn.addEventListener('click', () => {
                void this.runRuleFromForm(this.editingRule!.rule_id, runBtn, resultContainer);
            });
            actions.appendChild(runBtn);
        }

        form.append(actions, resultContainer);
        shell.appendChild(form);
        this.appendChild(shell);
    }

    private buildTrigger(form: HTMLFormElement): TriggerKind {
        const kind = (form.elements.namedItem('trigger') as HTMLSelectElement).value as TriggerKind['kind'];
        if (kind === 'cron') {
            const expression = (form.elements.namedItem('cron_expression') as HTMLInputElement).value.trim();
            return { kind: 'cron', expression };
        }
        if (kind === 'interval') {
            const seconds = parseInt((form.elements.namedItem('interval_seconds') as HTMLInputElement).value, 10);
            return { kind: 'interval', seconds: isNaN(seconds) ? 60 : seconds };
        }
        if (kind === 'on_type_assign') {
            const type_name = (form.elements.namedItem('type_name') as HTMLInputElement).value.trim() || null;
            return { kind: 'on_type_assign', type_name };
        }
        if (kind === 'on_type_unassign') {
            const type_name = (form.elements.namedItem('type_name') as HTMLInputElement).value.trim() || null;
            return { kind: 'on_type_unassign', type_name };
        }
        if (kind === 'on_attribute_set') {
            const attribute_key = (form.elements.namedItem('attribute_key') as HTMLInputElement).value.trim() || null;
            return { kind: 'on_attribute_set', attribute_key };
        }
        return { kind } as TriggerKind;
    }

    private async saveRule(form: HTMLFormElement, resultContainer: HTMLElement): Promise<void> {
        const name = (form.elements.namedItem('name') as HTMLInputElement).value.trim();
        const description = (form.elements.namedItem('description') as HTMLTextAreaElement).value.trim();
        const script = (form.elements.namedItem('script') as HTMLTextAreaElement).value;
        const enabled = (form.elements.namedItem('enabled') as HTMLInputElement).checked;
        const trigger = this.buildTrigger(form);

        const saveBtn = form.querySelector('button[type="submit"]') as HTMLButtonElement;
        saveBtn.disabled = true;
        resultContainer.innerHTML = '';

        try {
            if (this.editingRule) {
                const dto: UpdateRuleDto = { name, description, trigger, script, enabled };
                await ruleApi.Update(this.editingRule.rule_id, dto);
            } else {
                const dto: AddRuleDto = { name, description, trigger, script, enabled };
                await ruleApi.Add(dto);
            }
            this.view = 'list';
            void this.render();
        } catch (err) {
            saveBtn.disabled = false;
            const p = document.createElement('p');
            p.className = 'tool-error';
            p.textContent = err instanceof Error ? err.message : 'Failed to save rule.';
            resultContainer.appendChild(p);
        }
    }

    private async runRuleFromForm(rule_id: string, btn: HTMLButtonElement, container: HTMLElement): Promise<void> {
        btn.disabled = true;
        btn.textContent = 'Running…';
        container.innerHTML = '';

        let result: RuleRunResult;
        try {
            result = await ruleApi.Run(rule_id);
        } catch (err) {
            btn.disabled = false;
            btn.textContent = 'Run Now';
            const p = document.createElement('p');
            p.className = 'tool-error';
            p.textContent = err instanceof Error ? err.message : 'Failed to run rule.';
            container.appendChild(p);
            return;
        }

        btn.disabled = false;
        btn.textContent = 'Run Now';

        const panel = document.createElement('div');
        panel.className = result.success ? 'run-result run-result-ok' : 'run-result run-result-error';
        const text = result.success
            ? (result.output ?? 'Script ran successfully — no output')
            : (result.error ?? 'Rule failed with no error message.');
        panel.textContent = text;

        const dur = document.createElement('div');
        dur.style.marginTop = '8px';
        dur.style.fontSize = '12px';
        dur.style.opacity = '0.7';
        dur.textContent = `ran in ${result.duration_ms}ms`;
        panel.appendChild(dur);

        container.appendChild(panel);
    }
}

customElements.define('rules-screen', RulesScreen);

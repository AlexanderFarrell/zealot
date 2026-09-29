export type TriggerKind =
    | { kind: 'on_item_create' }
    | { kind: 'on_item_update' }
    | { kind: 'on_item_delete' }
    | { kind: 'on_comment_add' }
    | { kind: 'on_type_assign';    type_name: string | null }
    | { kind: 'on_type_unassign';  type_name: string | null }
    | { kind: 'on_attribute_set';  attribute_key: string | null }
    | { kind: 'cron';              expression: string }
    | { kind: 'interval';          seconds: number }
    | { kind: 'manual' };

export type TriggerKindTag = TriggerKind['kind'];

export const TRIGGER_KIND_LABELS: Record<TriggerKindTag, string> = {
    on_item_create:   'On Item Created',
    on_item_update:   'On Item Updated',
    on_item_delete:   'On Item Deleted',
    on_comment_add:   'On Comment Added',
    on_type_assign:   'On Type Assigned',
    on_type_unassign: 'On Type Unassigned',
    on_attribute_set: 'On Attribute Set',
    cron:             'Scheduled (Cron)',
    interval:         'Scheduled (Interval)',
    manual:           'Manual',
};

export class Rule {
    constructor(
        public readonly rule_id:     string,
        public readonly account_id:  string,
        public readonly name:        string,
        public readonly description: string,
        public readonly trigger:     TriggerKind,
        public readonly script:      string,
        public readonly enabled:     boolean,
        public readonly created_at:  string,
        public readonly last_run_at: string | null,
        public readonly last_error:  string | null,
        public readonly last_output: string | null,
    ) {}

    static fromDto(dto: RuleDto): Rule {
        return new Rule(
            dto.rule_id,
            dto.account_id,
            dto.name,
            dto.description,
            dto.trigger,
            dto.script,
            dto.enabled,
            dto.created_at,
            dto.last_run_at,
            dto.last_error,
            dto.last_output,
        );
    }
}

export interface RuleDto {
    rule_id:     string;
    account_id:  string;
    name:        string;
    description: string;
    trigger:     TriggerKind;
    script:      string;
    enabled:     boolean;
    created_at:  string;
    last_run_at: string | null;
    last_error:  string | null;
    last_output: string | null;
}

export interface AddRuleDto {
    name:         string;
    description?: string;
    trigger:      TriggerKind;
    script:       string;
    enabled?:     boolean;
}

export interface UpdateRuleDto {
    name?:        string;
    description?: string;
    trigger?:     TriggerKind;
    script?:      string;
    enabled?:     boolean;
}

export interface RuleRunResult {
    rule_id:     string;
    success:     boolean;
    output:      string | null;
    error:       string | null;
    duration_ms: number;
}

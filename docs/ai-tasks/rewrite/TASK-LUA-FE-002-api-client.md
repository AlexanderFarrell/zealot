# TASK-LUA-FE-002: `RuleAPI` client implementation

## Context

`packages/api/src/rule.ts` is currently an empty stub. This task implements the full API client following the exact pattern of `ItemAPI` and `CommentAPI`.

## Goal

Implement `RuleAPI` with methods for all six backend endpoints.

## Requirements

### `packages/api/src/rule.ts`

```typescript
import { AddRuleDto, Rule, RuleDto, RuleRunResult, UpdateRuleDto } from '@zealot/domain';

export class RuleAPI {
    private base: string;

    constructor(base: string = '/api') {
        this.base = base;
    }

    async GetAll(): Promise<Rule[]> {
        const resp = await fetch(`${this.base}/rule`);
        if (!resp.ok) throw new Error(await resp.text());
        const dtos: RuleDto[] = await resp.json();
        return dtos.map(Rule.fromDto);
    }

    async GetById(rule_id: string): Promise<Rule> {
        const resp = await fetch(`${this.base}/rule/${rule_id}`);
        if (!resp.ok) throw new Error(await resp.text());
        return Rule.fromDto(await resp.json());
    }

    async Add(dto: AddRuleDto): Promise<Rule> {
        const resp = await fetch(`${this.base}/rule`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(dto),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return Rule.fromDto(await resp.json());
    }

    async Update(rule_id: string, dto: UpdateRuleDto): Promise<Rule> {
        const resp = await fetch(`${this.base}/rule/${rule_id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(dto),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return Rule.fromDto(await resp.json());
    }

    async Delete(rule_id: string): Promise<void> {
        const resp = await fetch(`${this.base}/rule/${rule_id}`, { method: 'DELETE' });
        if (!resp.ok) throw new Error(await resp.text());
    }

    async Run(rule_id: string): Promise<RuleRunResult> {
        const resp = await fetch(`${this.base}/rule/${rule_id}/run`, { method: 'POST' });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }
}
```

### Register in `packages/api/src/index.ts`

Verify that `ZealotAPI` already imports `RuleAPI` (it should — it was stubbed). If the import exists but `this.Rule` is not instantiated, fix the instantiation.

## Dependencies

- TASK-LUA-FE-001

## Files to modify

- `packages/api/src/rule.ts`
- `packages/api/src/index.ts` (verify wiring)

## Verification

```bash
cd packages/api && npx tsc --noEmit
```

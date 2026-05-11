import { delete_req, get_json, patch_json, post_json, post_req } from "@websoil/engine";
import { Rule } from "@zealot/domain/src/rule";
import type { AddRuleDto, RuleDto, RuleRunResult, UpdateRuleDto } from "@zealot/domain/src/rule";
import { BaseAPI } from "./common";

export class RuleAPI extends BaseAPI {
    async GetAll(): Promise<Rule[]> {
        const dtos: RuleDto[] = await get_json(`${this.baseUrl}/rule`);
        return dtos.map(Rule.fromDto);
    }

    async GetById(rule_id: string): Promise<Rule> {
        const dto: RuleDto = await get_json(`${this.baseUrl}/rule/${rule_id}`);
        return Rule.fromDto(dto);
    }

    async Add(dto: AddRuleDto): Promise<Rule> {
        const result: RuleDto = await post_json(`${this.baseUrl}/rule`, dto) as RuleDto;
        return Rule.fromDto(result);
    }

    async Update(rule_id: string, dto: UpdateRuleDto): Promise<Rule> {
        const result: RuleDto = await patch_json(`${this.baseUrl}/rule/${rule_id}`, dto) as RuleDto;
        return Rule.fromDto(result);
    }

    async Delete(rule_id: string): Promise<void> {
        await delete_req(`${this.baseUrl}/rule/${rule_id}`);
    }

    async Run(rule_id: string): Promise<RuleRunResult> {
        const resp = await post_req(`${this.baseUrl}/rule/${rule_id}/run`, {});
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }
}

import { DateTime } from "luxon";
import { BaseAPI } from "./common";
import { RepeatEntry } from "@zealot/domain/src/repeat";
import type { RepeatEntryDto, UpdateRepeatEntryDto } from "@zealot/domain/src/repeat";
import { get_json, patch_json } from "@websoil/engine";

export class RepeatAPI extends BaseAPI {
    public async GetForDay(date: DateTime): Promise<RepeatEntry[]> {
        const iso = date.toISODate();
        let dtos: RepeatEntryDto[] = await get_json(`${this.baseUrl}/repeat/day/${iso}`);
        return dtos.map(d => new RepeatEntry(d));
    }

    public async SetStatus(dto: UpdateRepeatEntryDto) {
        const iso = dto.date.toISODate();
        let data: Record<string, any> = {};
        if (dto.status) {
            data['status'] = dto.status!;
        }
        if (dto.comment) {
            data['comment'] = dto.comment!;
        }

        return patch_json(`${this.baseUrl}/repeat/${dto.item_id}/day/${iso}`, {
            status: dto.status,
            comment: dto.comment
        })
    }
}

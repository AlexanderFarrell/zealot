import { DateTime } from "luxon";
import { get_json } from "@websoil/engine";
import { Item } from "@zealot/domain/src/item";
import type { ItemDto } from "@zealot/domain/src/item";
import { BaseAPI } from "./common";

export class PlannerAPI extends BaseAPI {
    public async GetForDay(date: DateTime): Promise<Item[]> {
        const iso = date.toISODate();
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/planner/day/${iso}`);
        return dtos.map((dto) => new Item(dto));
    }

    public async GetForWeek(date: DateTime): Promise<Item[]> {
        const week = date.toFormat("kkkk-'W'WW");
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/planner/week/${week}`);
        return dtos.map((dto) => new Item(dto));
    }

    public async GetForMonth(date: DateTime): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(
            `${this.baseUrl}/planner/month/${date.month}/year/${date.year}`,
        );
        return dtos.map((dto) => new Item(dto));
    }

    public async GetForYear(date: DateTime): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/planner/year/${date.year}`);
        return dtos.map((dto) => new Item(dto));
    }
}

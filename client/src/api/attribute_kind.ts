import { BasicAPI, patch_json } from "../core/api_helper";
import { events } from "../core/events";

export const AttributeKindsBaseTypes = [
    'text',
    'integer',
    'decimal',
    'date',
    'week',
    'dropdown',
    'boolean',
    'list'
]

export interface AttributeKind {
    kind_id?: number,
    key: string,
    description: string,
    base_type: string,
    config: any,
    is_system?: boolean,
}

class AttributeKindAPIHandler extends BasicAPI<AttributeKind> {
    constructor() {
        super(`/api/item/kind`)
    }

    async update_conifg(id: number, config: any) {
        return patch_json(`${this.URL}/${id}/config`, {config})
    }
}

// Store attribute kinds for validation
let kinds: AttributeKind[] | null = null;
let kinds_in_flight: Promise<AttributeKind[]> | null = null;


events.on('refresh_attributes', () => {
    // Mark stale
    kinds = null;
    if (!kinds_in_flight) {
        kinds_in_flight = AttributeKindAPI.get_all().then(data => {
            kinds = data;
            kinds_in_flight = null;
            return kinds;
        }).catch(err => {
            kinds_in_flight = null;
            throw err;
        })
    }
})

export async function get_attribute_kinds(force_refresh: boolean = false) {
    if (!force_refresh && kinds) {
        return kinds;
    }

    if (kinds_in_flight) {
        return kinds_in_flight;
    }

    kinds_in_flight = AttributeKindAPI.get_all().then(data => {
        kinds = data;
        kinds_in_flight = null;
        return kinds;
    }).catch(err => {
        kinds_in_flight = null;
        throw err;
    })

    return kinds_in_flight;
}

export async function get_kind_for_key(key: string) {
    const data = await get_attribute_kinds();
    return data.find(e => e.key === key);
}

export const AttributeKindAPI = new AttributeKindAPIHandler();

export default AttributeKindAPI;
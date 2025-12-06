import { BasicAPI } from "../core/api_helper";

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
}

export const AttributeKindAPI = new AttributeKindAPIHandler();

export default AttributeKindAPI;
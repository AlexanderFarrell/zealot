import { Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item, ItemRelationship } from '@zealot/domain/src/item';
import { ItemTypeRef } from '@zealot/domain/src/item_type';
import { isBlankAttributeValue } from './attribute_value_input';

const itemApi = new ItemAPI('/api');
const attrApi = new AttributeAPI('/api');

export async function saveTitle(
    item: Item,
    input: HTMLInputElement,
): Promise<'saved' | 'unchanged' | 'failed'> {
    const nextTitle = input.value.trim();
    const previousTitle = item.Title;

    if (nextTitle === previousTitle || !nextTitle) {
        input.value = previousTitle;
        return 'unchanged';
    }

    try {
        const updated = await itemApi.Update(item.ItemID, {
            item_id: item.ItemID,
            title: nextTitle,
        });
        item.Title = updated.Title;
        return 'saved';
    } catch (error) {
        Popups.add_error((error as Error).message ?? 'Failed to save title.');
        input.value = previousTitle;
        return 'failed';
    }
}

export async function saveTypes(
    item: Item,
    nextTypes: string[],
): Promise<'saved' | 'unchanged' | 'failed'> {
    const previousTypes = item.Types.map((typeRef) => typeRef.Name);
    const added = nextTypes.filter((name) => !previousTypes.includes(name));
    const removed = previousTypes.filter((name) => !nextTypes.includes(name));

    if (added.length === 0 && removed.length === 0) return 'unchanged';

    try {
        await Promise.all([
            ...added.map((name) => itemApi.AssignType(item.ItemID, name)),
            ...removed.map((name) => itemApi.UnassignType(item.ItemID, name)),
        ]);
        item.Types = nextTypes.map((name) => new ItemTypeRef({ is_system: false, name, type_id: -1 }));
        return 'saved';
    } catch (error) {
        Popups.add_error((error as Error).message ?? 'Failed to update item types.');
        return 'failed';
    }
}

export async function saveAttribute(
    item: Item,
    key: string,
    value: unknown,
    kind: AttributeKind | undefined,
): Promise<'saved' | 'removed' | 'failed'> {
    try {
        if (isBlankAttributeValue(value, kind)) {
            await attrApi.remove(item.ItemID, key);
            return 'removed';
        }
        await attrApi.set_value(item.ItemID, key, value);
        return 'saved';
    } catch (error) {
        Popups.add_error((error as Error).message ?? `Failed to save "${key}".`);
        return 'failed';
    }
}

export async function createItem(params: {
    attributes: Record<string, unknown>;
    contextItemId?: number;
    relationship?: ItemRelationship;
    title: string;
    types: string[];
}): Promise<Item> {
    const addDto: {
        attributes?: Record<string, unknown>;
        content: string;
        links?: Array<{ other_item_id: number; relationship: ItemRelationship }>;
        title: string;
        types?: string[];
    } = {
        content: '',
        title: params.title,
    };

    if (Object.keys(params.attributes).length > 0) {
        addDto.attributes = params.attributes;
    }

    if (params.contextItemId != null) {
        addDto.links = [{
            other_item_id: params.contextItemId,
            relationship: params.relationship ?? 'parent',
        }];
    }

    if (params.types.length > 0) {
        addDto.types = params.types;
    }

    return itemApi.Add(addDto);
}

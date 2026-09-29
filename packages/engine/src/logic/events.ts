import mitt from 'mitt';

export const Events = mitt();
export const ItemEvents = {
    created: 'item:created',
    deleted: 'item:deleted',
} as const;

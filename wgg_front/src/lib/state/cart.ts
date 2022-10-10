import { CartCurrentQueryDocument, CartFragment, Provider } from '$lib/api/graphql_types';
import { Client, queryStore } from '$lib/api/urql';
import { derived, Readable } from 'svelte/store';

export type ProductId = string;
export type CartStore = Readable<CartData | undefined>;

/**
 * Initialise the global cart store, returns it to be passed to $page.data
 */
export function initialiseCart(client: Client): CartStore {
    const store = queryStore({ query: CartCurrentQueryDocument, client });

    return derived(store, (x, set) => {
        if (!x.fetching) {
            if (x.data) {
                const cartInfo = x.data.cartCurrent;

                set({
                    getProductQuantity: (provider, productId) => getProductQuantityImpl(cartInfo, provider, productId),
                    ...cartInfo
                });
            } else {
                set(undefined);
            }
        }
    });
}

export interface CartQuery {
    type: 'RawProduct' | 'Note' | 'AggregateIngredient';
}

export interface CartData extends CartFragment {
    getProductQuantity(provider: Provider, productId: ProductId): QuantityInfo[];
}

export interface QuantityInfo {
    quantity: number;
    origin: 'Indirect' | 'Direct';
}

function getProductQuantityImpl(cart: CartFragment, provider: Provider, productId: ProductId): QuantityInfo[] {
    const results: QuantityInfo[] = cart.contents
        .filter((x) => x.__typename != 'CartNoteProduct')
        .map((x) => {
            if (x.__typename == 'CartAggregateProduct') {
                // Have to search constituent ingredients.
                const item = x.aggregate.ingredients.find((y) => y.id == productId && y.provider == provider);

                if (item != undefined) {
                    return {
                        quantity: x.quantity,
                        origin: 'Indirect'
                    };
                }
            } else if (x.__typename == 'CartProviderProduct') {
                if (x.product.id == productId && x.product.provider == provider) {
                    return {
                        quantity: x.quantity,
                        origin: 'Direct'
                    };
                }
            }

            return undefined;
        })
        .filter((x): x is QuantityInfo => x != undefined);

    return results;
}

// export interface CartStore extends Readable<CartFragment> {
//     send(notification: Notification): void;

//     remove(i: number): void;

//     error(msg: string, title?: string, timeout?: number): void;

//     warning(msg: string, title?: string, timeout?: number): void;

//     info(msg: string, title?: string, timeout?: number): void;

//     success(msg: string, title?: string, timeout?: number): void;
// }

// function createNotificationStore(): NotificationStore {
//     const backing_store: Writable<Notification[]> = writable([]);

//     const send = (notification: Notification) => {
//         backing_store.update((state) => {
//             return [...state, notification];
//         });
//     };

//     const remove = (index: number) => {
//         backing_store.update((state) => {
//             state.splice(index, 1);
//             return state;
//         });
//     };

//     const derived_notifications: Readable<Notification[]> = derived(backing_store, ($_notifications, set) => {
//         // Set the value to our normal store's value.
//         set($_notifications);

//         if ($_notifications.length > 0) {
//             // Set timeouts one at a time
//             const timeout = setTimeout(() => {
//                 backing_store.update((state) => {
//                     state.shift();
//                     return state;
//                 });
//             }, $_notifications[0].timeToLiveMs);

//             // Clear timeouts in case of destruction
//             return () => {
//                 clearTimeout(timeout);
//             };
//         }
//     });

//     const { subscribe } = derived_notifications;

//     return {
//         subscribe,
//         send,
//         remove,
//         error: (msg: string, title: string = 'Error', timeout: number = NOTIFICATION_TIMEOUT) =>
//             send(new Notification(NotificationType.Error, msg, title, timeout)),
//         warning: (msg: string, title: string = 'Warning', timeout: number = NOTIFICATION_TIMEOUT) =>
//             send(new Notification(NotificationType.Warning, msg, title, timeout)),
//         info: (msg: string, title: string = 'Info', timeout: number = NOTIFICATION_TIMEOUT) =>
//             send(new Notification(NotificationType.Info, msg, title, timeout)),
//         success: (msg: string, title: string = 'Success', timeout: number = NOTIFICATION_TIMEOUT) =>
//             send(new Notification(NotificationType.Success, msg, title, timeout))
//     };
// }

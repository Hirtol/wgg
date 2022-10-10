import { CartFragment } from "$lib/api/graphql_types";
import { Readable } from "svelte/store";

export let cart: Readable<CartFragment> | undefined;

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
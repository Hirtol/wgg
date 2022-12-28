import { ModalComponent, ModalSettings, modalStore } from '@skeletonlabs/skeleton';

export { modalStore };
export type { ModalComponent, ModalSettings };

/**
 * Trigger a new modal, where one can optionally push it to either the front/back.
 */
export function triggerModal(modal: ModalSettings, addToFront: boolean = true) {
    if (addToFront) {
        modalStore.update((queue) => {
            queue.unshift(modal);
            return queue;
        });
    } else {
        modalStore.trigger(modal);
    }
}

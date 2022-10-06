/* eslint-disable @typescript-eslint/no-empty-function */
import { derived, Readable, writable, Writable } from 'svelte/store';

export const globalLoading = createGlobalProgressStore();

export enum Progress {
    Begin = 'Begin',
    Starting = 'Starting',
    Loading = 'Loading',
    Idle = 'Idle'
}

export interface GlobalProgress extends Readable<Progress> {
    /**
     * Submit an arbitrary promise, and activate the global progress bar until it is resolved.
     *
     * Returns the same promise as provided to allow easy chaining.
     */
    submit<T = never>(promise: Promise<T>): Promise<T>;

    /**
     * Manually start the progress bar, needs to be manually stopped with a call to `stop()`.
     */
    start(): void;

    /**
     * Manually stops the progress bar.
     */
    stop(): void;
}

function createGlobalProgressStore(): GlobalProgress {
    const backing_store: Writable<Promise<never> | undefined> = writable(undefined);

    const submit = (promise: Promise<never>) => {
        backing_store.set(promise);
        return promise;
    };

    const derived_progress: Readable<Progress> = derived(backing_store, (promise, set) => {
        if (promise != undefined) {
            set(Progress.Begin);
            // Add a small delay to allow the progress bar to move to starting
            setTimeout(() => set(Progress.Starting), 1);

            promise.finally(() => {
                backing_store.set(undefined);
            });
        } else {
            set(Progress.Idle);
        }
    });

    const { subscribe } = derived_progress;

    return {
        subscribe,
        submit,
        start: () => {},
        stop: () => {}
    };
}

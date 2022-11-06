/* eslint-disable @typescript-eslint/no-empty-function */
import { derived, type Readable, writable, type Writable } from 'svelte/store';

/**
 * The global store for tracking some kind of progress
 */
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
    submit<T = unknown>(promise: Promise<T>): Promise<T>;

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
    const backing_store: Writable<Promise<unknown> | undefined> = writable(undefined);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const submit = (promise: Promise<any>) => {
        backing_store.set(promise);
        return promise;
    };
    const start = () => {
        const promise: Promise<unknown> = new Promise((resolver) => (startPromiseResolver = resolver));
        submit(promise);
    };
    const stop = () => {
        if (startPromiseResolver) {
            startPromiseResolver({});
        }
    };

    const derived_progress: Readable<Progress> = derived(backing_store, (promise, set) => {
        if (promise != undefined) {
            set(Progress.Begin);
            // Add a small delay to allow the progress bar to move to starting
            const timeout = setTimeout(() => set(Progress.Starting), 0);

            promise.finally(() => {
                clearTimeout(timeout);
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
        start,
        stop
    };
}

let startPromiseResolver: ((value: unknown) => void) | undefined = undefined;

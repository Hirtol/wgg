import { Writable, writable, Readable, readable, Subscriber } from 'svelte/store';

export const isMobileScreen: Readable<boolean> = readable(false, (set: Subscriber<boolean>) => {
    // Initialise the value
    onResize();
    window.addEventListener('resize', onResize);

    function onResize() {
        set(window.matchMedia('only screen and (max-width: 760px)').matches);
    }

    return () => {
        window.removeEventListener('resize', onResize);
    };
});


/**
 *
 * @param key The key which is used in local storage to get the value
 * @param startValue The initial value, if a local value is already saved this will be ignored.
 * @returns A writable store which will immediately persist any written value to localStorage.
 */
 export function createPersistentWritable<T>(key: string, startValue?: T): Writable<T> {
    if (typeof window === 'undefined') {
        return writable();
    }

    const item = localStorage.getItem(key);
    if (item) {
        startValue = Object.assign({}, startValue,JSON.parse(item));
    }

    const store = writable(startValue);

    store.subscribe((current) => localStorage.setItem(key, JSON.stringify(current)));

    return store;
}
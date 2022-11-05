import { Writable, writable } from 'svelte/store';

export { isMobileScreen } from '../utils';

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
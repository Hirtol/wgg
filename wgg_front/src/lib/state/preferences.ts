import { Provider } from '$lib/api/graphql_types';
import { Writable } from 'svelte/store';
import { createPersistentWritable } from './stores';

export type DisplayPriceOptions = Provider | 'AVERAGE' | 'MAX' | 'MIN';

export interface Preferences {
    /**
     * The price to display by default in/on the cart overview.
     *
     * Most providers will have their own respective tabs in the overview, but for a quick glance it is useful to have
     * a general price to look at.
     */
    displayPrice: DisplayPriceOptions;
}

export function createPreferenceStore(): Writable<Preferences> {
    const defaultItem: Preferences = {
        displayPrice: 'AVERAGE'
    };

    const store = createPersistentWritable('wggPreferences', defaultItem);

    return store;
}

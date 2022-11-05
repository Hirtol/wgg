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

    /**
     * The provider which is selected by default when entering new pages.
     */
    favoriteProvider: Provider;
}

export function createPreferenceStore(): Writable<Preferences> {
    const defaultItem: Preferences = {
        displayPrice: 'AVERAGE',
        favoriteProvider: Provider.Picnic
    };

    const store = createPersistentWritable('wggPreferences', defaultItem);

    return store;
}

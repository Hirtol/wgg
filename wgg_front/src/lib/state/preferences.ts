import { PriceFilter, Provider } from '$lib/api/graphql_types';
import { notifications } from '$lib/components/notifications/notification';
import { getContext, setContext } from 'svelte';
import { get, Writable } from 'svelte/store';
import { getProviders, ProviderMap } from './providers';
import { createPersistentWritable } from './stores';

export type PreferenceStore = Writable<Preferences>;
export type DisplayPriceOptions = typeof DisplayPriceOptionsConcrete;

export const DisplayPriceOptionsConcrete = ['FAVORITE PROVIDER', 'AVERAGE', 'MAX', 'MIN'] as const;

export interface Preferences {
    /**
     * The price to display by default in/on the cart overview.
     *
     * Most providers will have their own respective tabs in the overview, but for a quick glance it is useful to have
     * a general price to look at.
     */
    displayPrice: DisplayPriceOptions;

    /**
     * The price to display by default for aggregate ingredients
     */
    aggregateDisplayPrice: PriceFilter;

    /**
     * The provider which is selected by default when entering new pages.
     */
    favouriteProvider: Provider;
}

/**
 * Whenever the web-app is started we should verify whether our previous favorites are still valid in the current server config!
 */
export function verifyPreferenceIntegrity(
    preferences: Writable<Preferences>,
    availableProviders: ProviderMap
): boolean {
    let isUnmodified = true;
    const globalProviders = getProviders();
    const currentPrefs = get(preferences);

    if (!availableProviders.has(currentPrefs.favouriteProvider)) {
        notifications.warning(
            `Favorite provider '${currentPrefs.favouriteProvider}' is no longer available on the server, resetting!`,
            'Preference Update'
        );
        // If there are no available providers then the server is misconfigured anyway...
        currentPrefs.favouriteProvider = availableProviders.keys().next().value;
        isUnmodified = false;
    }

    // Check if the displayPrice is for a specific Provider, and if so, whether that Provider is available.
    if (
        globalProviders.includes(currentPrefs.displayPrice as Provider) &&
        !availableProviders.has(currentPrefs.displayPrice as Provider)
    ) {
        notifications.warning(
            `Display Price '${currentPrefs.displayPrice}' is no longer available on the server, resetting!`,
            'Preference Update'
        );
        currentPrefs.displayPrice = 'AVERAGE';
        isUnmodified = false;
    }

    preferences.set(currentPrefs);

    return isUnmodified;
}

export function createPreferenceStore(): Writable<Preferences> {
    const defaultItem: Preferences = {
        displayPrice: 'AVERAGE',
        aggregateDisplayPrice: PriceFilter.Average,
        favouriteProvider: Provider.Picnic
    };

    const store = createPersistentWritable('wggPreferences', defaultItem);

    return store;
}

export function setContextPreference(preferenceStore: PreferenceStore) {
    setContext('wgg_preference', preferenceStore);
}

export function getContextPreferences(): PreferenceStore {
    return getContext('wgg_preference');
}

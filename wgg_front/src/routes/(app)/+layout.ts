import { PriceFilter, ProviderInfo, ViewerContextFragment, ViewerInfoQueryDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { authSession, createAvailableProvidersMap, initialiseRealCart, verifyPreferenceIntegrity } from '$lib/state';
import { redirect } from '@sveltejs/kit';
import { Client } from '@urql/svelte';
import { get } from 'svelte/store';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async (event) => {
    // First check if we just had a login event, if not we proceed with the manual check.
    const { client, preferences } = await event.parent();
    const concPrefs = get(preferences);
    let cartContents = undefined;
    let providers = undefined;

    const { isAuthenticated, user, remoteProviders } = await authenticateUser(client, concPrefs.aggregateDisplayPrice);

    // Perform authentication check
    if (!isAuthenticated) {
        const loginUrl = event.url.href.length > 0 ? `/login?redirect=${event.url.pathname}` : '/login';

        throw redirect(302, loginUrl);
    } else {
        cartContents = user?.currentCart;
        providers = remoteProviders;
    }

    // Verify preference integrity
    const availableProviders = createAvailableProvidersMap(providers ?? []);
    if (verifyPreferenceIntegrity(preferences, availableProviders)) {
        console.log('Preferences verified');
    }

    // We can safely assume we're authenticated.
    return {
        cart: initialiseRealCart(client, preferences, cartContents),
        availableProviders
    };
};

/**
 * Authenticate the user, if possible.
 *
 * If the current user is authenticated, the `authSession` and `isUserAuthenticated` stores will be initialised.
 */
async function authenticateUser(
    client: Client,
    aggregatePrice: PriceFilter
): Promise<{
    user: ViewerContextFragment | undefined;
    isAuthenticated: boolean;
    remoteProviders: ProviderInfo[] | undefined;
}> {
    try {
        const { item } = await asyncQueryStore({
            query: ViewerInfoQueryDocument,
            variables: { price: aggregatePrice },
            client
        });
        authSession.set(item.data?.viewer);

        if (item.error) {
            console.log('User is not authenticated', item.error);
        }

        return {
            user: item.data?.viewer,
            isAuthenticated: item.data != undefined,
            remoteProviders: item.data?.proProviders
        };
    } catch (error) {
        return { user: undefined, isAuthenticated: false, remoteProviders: undefined };
    }
}

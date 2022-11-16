import { GetFilteredPromotionsDocument, Provider } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { verifyProviderOrError } from '$lib/loading';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client, preferences } = await event.parent();

    const r_prefs = get(preferences);
    // We want to preserve history when bouncing back and forth between pages. At the moment searchParams are the easiest way of doing that.
    const initialProvider = verifyProviderOrError(event.url.searchParams.get('provider') ?? r_prefs.favouriteProvider);

    const { store } = await asyncQueryStore({
        query: GetFilteredPromotionsDocument,
        client: client,
        variables: { filters: { provider: initialProvider as Provider } },
        requestPolicy: 'cache-first'
    });

    return {
        result: store,
        initialProvider
    };
};

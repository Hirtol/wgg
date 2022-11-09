import { GetFilteredPromotionsDocument, Provider } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { verifyProviderOrError } from '$lib/loading';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client, preferences } = await event.parent();

    const r_prefs = get(preferences);
    // We want to preserve history when bouncing back and forth between pages. At the moment searchParams are the easiest way of doing that.
    const initialProvider = verifyProviderOrError(event.url.searchParams.get('provider') ?? r_prefs.favoriteProvider);

    // The request-policy would ideally be 'cache-first', however URQL strangely returns only the explicitly declared parts of an interface.
    // This causes issues in WggSaleCategory->Items, as the returned item from the cache only contains `typename` and `id`, but none of the type-specific stuff from
    // either `WggSaleGroup` or `WggSearchProduct`, causing errors.
    const { store } = await asyncQueryStore({
        query: GetFilteredPromotionsDocument,
        client: client,
        variables: { filters: { provider: initialProvider as Provider } },
        requestPolicy: 'network-only'
    });

    return {
        result: store,
        initialProvider
    };
};

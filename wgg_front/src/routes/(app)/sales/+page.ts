import { GetFilteredPromotionsDocument, Provider } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client, preferences } = await event.parent();

    const r_prefs = get(preferences);
    // We want to preserve history when bouncing back and forth between pages. At the moment searchParams are the easiest way of doing that.
    const initialProvider = event.url.searchParams.get('provider') ?? r_prefs.favoriteProvider;

    // We provide `initialProvider` without parsing it. Technically a user could mess about and this would error out as a result.
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

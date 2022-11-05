import { GetFilteredPromotionsDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client, preferences } = await event.parent();

    console.log(event.url.searchParams.get('provider'));

    const r_prefs = get(preferences);

    const { store } = await asyncQueryStore({
        query: GetFilteredPromotionsDocument,
        client: client,
        variables: { filters: { provider: r_prefs.favoriteProvider } }
    });

    return {
        result: store
    };
};

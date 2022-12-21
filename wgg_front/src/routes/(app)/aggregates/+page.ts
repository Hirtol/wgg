import { GetAggregateIngredientsDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client, preferences } = await event.parent();
    const r_prefs = get(preferences);

    const { store } = await asyncQueryStore({
        query: GetAggregateIngredientsDocument,
        variables: { price: r_prefs.aggregateDisplayPrice },
        client: client
    });

    return {
        store,
        title: "Wgg - Aggregates"
    };
};

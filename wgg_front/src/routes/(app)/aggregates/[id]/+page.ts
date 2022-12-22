import { GetAggregateIngredientDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';

export const load = (async (event) => {
    const { client, preferences } = await event.parent();
    const r_prefs = get(preferences);

    const { store, item } = await asyncQueryStore({
        query: GetAggregateIngredientDocument,
        variables: { id: +event.params.id, price: r_prefs.aggregateDisplayPrice },
        client: client
    });

    return {
        store,
        title: `Wgg - ${item.data?.aggregateIngredient.name} (Aggregate Ingredient)`
    };
}) satisfies PageLoad;

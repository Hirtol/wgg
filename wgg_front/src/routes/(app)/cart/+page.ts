import { CartCurrentQueryDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import type { PageLoad } from './$types';

export const load: PageLoad = async (data) => {
    const { client } = await data.parent();
    // Force refresh cart data just in case. The global cart data will be indirectly refreshed by this query.
    const { store } = await asyncQueryStore({
        query: CartCurrentQueryDocument,
        client
    });

    return {
        result: store
    };
};

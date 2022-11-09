import { GetSaleSublistDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { verifyProviderOrError } from '$lib/loading';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client } = await event.parent();
    const provider = verifyProviderOrError(event.params.provider);

    const { store: listStore } = await asyncQueryStore({
        query: GetSaleSublistDocument,
        variables: { id: event.params.listId, provider },
        client: client
    });

    return {
        listStore
    };
};

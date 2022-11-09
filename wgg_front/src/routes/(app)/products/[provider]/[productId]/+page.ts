import { FullProductQueryDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { verifyProviderOrError } from '$lib/loading';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client } = await event.parent();
    const provider = verifyProviderOrError(event.params.provider);

    const { store } = await asyncQueryStore({
        query: FullProductQueryDocument,
        variables: { productId: event.params.productId, provider },
        client: client
    });

    return {
        store
    };
};

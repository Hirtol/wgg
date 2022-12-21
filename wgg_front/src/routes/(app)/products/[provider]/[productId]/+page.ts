import { FullProductQueryDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { verifyProviderOrError } from '$lib/loading';
import { capitaliseFirst } from '$lib/utils';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client } = await event.parent();
    const provider = verifyProviderOrError(event.params.provider);

    const { store, item } = await asyncQueryStore({
        query: FullProductQueryDocument,
        variables: { productId: event.params.productId, provider },
        client: client
    });

    return {
        store,
        title: `Wgg - ${item.data?.proProduct.name ?? "Unknown Product"} (${capitaliseFirst(item.data?.proProduct.providerInfo.provider ?? '')})`
    };
};

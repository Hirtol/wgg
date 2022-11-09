import { FullProductQueryDocument, Provider } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { getProviders } from '$lib/state';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client } = await event.parent();

    if (getProviders().includes(event.params.provider as Provider)) {
        const provider = event.params.provider as Provider;
        const { store } = await asyncQueryStore({
            query: FullProductQueryDocument,
            variables: { productId: event.params.productId, provider },
            client: client
        });

        return {
            store
        };
    } else {
        console.error('Invalid `Provider` provided to front-end!');

        throw error(401, {
            code: '401',
            message: `Invalid Provider: ${event.params.provider}`
        });
    }
};

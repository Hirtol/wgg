import { createUrqlClient } from '$lib/api/urql';
import { LayoutLoad } from './$types';
import { readable } from 'svelte/store';

/**
 * Create the `URQL` client used throughout the application.
 */
export const load: LayoutLoad = async (event) => {
    const client = createUrqlClient({ url: '/api/graphql', fetch: event.fetch, requestPolicy: 'cache-and-network' });

    return {
        client,
        cart: readable()
    };
};

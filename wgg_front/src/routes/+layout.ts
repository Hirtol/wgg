import { createUrqlClient } from '$lib/api/urql';
import { LayoutLoad } from './$types';
import { createPreferenceStore } from '$lib/state';

export const preprocess = false;
export const ssr = false;

/**
 * Create the `URQL` client used throughout the application.
 */
export const load: LayoutLoad = async (event) => {
    const client = createUrqlClient({ url: '/api/graphql', fetch: event.fetch, requestPolicy: 'cache-and-network' });
    const preferences = createPreferenceStore();

    return {
        client,
        preferences,
    };
};

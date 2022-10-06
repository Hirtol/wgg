import { createUrqlClient } from '$lib/api/urql';
import type { LayoutLoad } from '$types';
import { Client } from '@urql/svelte';

/**
 * Create the `URQL` client used throughout the application.
 */
export const load: LayoutLoad<{client: Client}> = async (event) => {
    const client = createUrqlClient({ url: '/api/graphql', fetch: event.fetch });

    return {
        client
    };
};


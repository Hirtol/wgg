import type { PageLoad } from './$types';

export const load: PageLoad = async (data) => {
    const { client, cart } = await data.parent();
    // Force refresh cart data just in case. The global cart data will be indirectly refreshed by this query.
    await cart.refreshContent(client);
    console.log('HEY');

    return {};
};

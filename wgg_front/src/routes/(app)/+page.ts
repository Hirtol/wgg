import { GetAllPromotionsDocument } from '$lib/api/graphql_types';
import { asyncQueryStore } from '$lib/api/urql';
import { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
    const { client } = await event.parent();
    const {store} = await asyncQueryStore({ query: GetAllPromotionsDocument, client: client });
    return {
        result: store
    };
};

import { GetAllPromotionsDocument, GetAllPromotionsQuery } from '$lib/api/graphql_types/graphql';
import { asyncQueryStore, OperationResultStore } from '$lib/api/urql';
import type { PageLoad } from '$types';

export const load: PageLoad<{ result: OperationResultStore<GetAllPromotionsQuery> }> = async (event) => {
    const { client } = await event.parent();
    const result = await asyncQueryStore({ query: GetAllPromotionsDocument, client: client });
    return {
        result
    };
};

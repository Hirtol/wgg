import { GetAllPromotionsDocument, type GetAllPromotionsQuery } from '$lib/api/graphql_types';
import { asyncQueryStore, type OperationResultStore } from '$lib/api/urql';
import type { PageLoad } from '.svelte-kit/types/src/routes/(app)/$types';

export const load: PageLoad<{ result: OperationResultStore<GetAllPromotionsQuery> }> = async (event) => {
    const { client } = await event.parent();
    const result = await asyncQueryStore({ query: GetAllPromotionsDocument, client: client });
    return {
        result
    };
};

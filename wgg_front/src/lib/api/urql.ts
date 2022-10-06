/* eslint-disable @typescript-eslint/no-explicit-any */
import { cacheExchange } from '@urql/exchange-graphcache';
import { requestPolicyExchange } from '@urql/exchange-request-policy';
import { notifications } from '$lib/components/notifications/notification';
import {
    Client,
    ClientOptions,
    createClient,
    dedupExchange,
    errorExchange,
    fetchExchange,
    AnyVariables,
    OperationResult,
    QueryArgs,
    queryStore,
    mutationStore,
    MutationArgs,
    SubscriptionHandler
} from '@urql/svelte';
import { Readable } from 'svelte/store';

export * from '@urql/svelte';

/**
 * Central point to create a Urql client with the exact settings that we want.
 * @param opts The options to pass to the client.
 * @returns A Urql client
 */
export function createUrqlClient(opts?: ClientOptions): Client {
    return createClient({
        url: '/api/graphql',
        ...opts,
        exchanges: [
            dedupExchange,
            errorExchange({
                onError: (error) => {
                    console.log(`Generic API Error:`, error);
                    notifications.error(error.message, 'Failed to query API');
                }
            }),
            requestPolicyExchange({ ttl: 3 * 60 * 1000 }),
            cacheExchange({
                keys: {
                    UnitQuantity: () => null
                },
                updates: {
                    Mutation: {}
                }
            }),
            fetchExchange
        ]
    });
}

/**
 * Same as the {@link queryStore} function, except that it returns a `Promise`, which resolves when the store has fetched its first data, or errored out.
 *
 * # Example
 *
 * ```ts
 * const result = await asyncQueryStore({query: HomePageQueryDocument, client: client});
 * ```
 */
export async function asyncQueryStore<Data = any, Variables extends AnyVariables = AnyVariables>(
    args: QueryArgs<Data, Variables>
): Promise<OperationResultStore<Data, Variables>> {
    const result = queryStore(args);

    let resolver: (value: any) => void;
    let rejector: (reason: any) => void;

    const finalPromise: Promise<OperationResultStore<Data, Variables>> = new Promise((accept, reject) => {
        resolver = accept;
        rejector = reject;
    });

    const unsubscribe = result.subscribe((x) => {
        if (x.data !== undefined) {
            resolver(result);
            unsubscribe();
        }

        if (!x.fetching) {
            if (x.error) {
                rejector(result);
                unsubscribe();
            }
        }
    });

    return finalPromise;
}

/**
 * Same as the {@link mutationStore} function, except that it returns a `Promise`, which resolves when the store has fetched its first data, or errored out.
 *
 * # Example
 *
 * ```ts
 * const result = await asyncMutationStore({query: HomePageMutationDocument, client: client});
 * ```
 */
 export async function asyncMutationStore<Data = any, Variables extends AnyVariables = AnyVariables>(
    args: MutationArgs<Data, Variables>, handler?: SubscriptionHandler<Data, Data>
): Promise<OperationResultStore<Data, Variables>> {
    const result = mutationStore(args, handler);

    let resolver: (value: any) => void;
    let rejector: (reason: any) => void;

    const finalPromise: Promise<OperationResultStore<Data, Variables>> = new Promise((accept, reject) => {
        resolver = accept;
        rejector = reject;
    });

    const unsubscribe = result.subscribe((x) => {
        if (x.data !== undefined) {
            resolver(result);
            unsubscribe();
        }

        if (!x.fetching) {
            if (x.error) {
                rejector(result);
                unsubscribe();
            }
        }
    });

    return finalPromise;
}

/* Types copied from Urql/svelte, but they were not exposed so you couldn't actually type your properties! */
export interface OperationResultState<Data = any, Variables extends AnyVariables = AnyVariables>
    extends OperationResult<Data, Variables> {
    fetching: boolean;
}

/** A Readable containing an `OperationResult` with a fetching flag. */
export type OperationResultStore<Data = any, Variables extends AnyVariables = AnyVariables> = Readable<
    OperationResultState<Data, Variables>
>;

/**
 * Sometimes, one can provide a singular value to the API, but they're not turned into an array in our methods here.
 * Therefore we'll have to do a manual check to ensure we don't use invalid data.
 *
 * @param ids Ids in the form of a single number or an array.
 * @returns An array form of the provided ids
 */
function numberOrArray(ids: any): number[] {
    if (Array.isArray(ids)) {
        return ids;
    } else {
        return [ids];
    }
}

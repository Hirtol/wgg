/* eslint-disable @typescript-eslint/no-explicit-any */
import { cacheExchange } from '@urql/exchange-graphcache';
import { requestPolicyExchange } from '@urql/exchange-request-policy';
import { notifications } from '$lib/components/notifications/notification';
import {
    Client,
    type ClientOptions,
    createClient,
    dedupExchange,
    errorExchange,
    fetchExchange,
    type AnyVariables,
    type OperationResult,
    type QueryArgs,
    queryStore,
    mutationStore,
    type MutationArgs,
    type SubscriptionHandler
} from '@urql/svelte';
import type { Readable } from 'svelte/store';
import { ProviderInfo, WggSaleCategory } from './graphql_types';

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
                    notifications.error(error.message, 'Failed to query API');
                }
            }),
            requestPolicyExchange({ ttl: 3 * 60 * 1000 }),
            cacheExchange({
                keys: {
                    UnitQuantity: () => null,
                    FreshLabel: () => null,
                    UnitPrice: () => null,
                    UnavailableItem: () => null,
                    SaleLabel: () => null,
                    SaleValidity: () => null,
                    PrepTime: () => null,
                    AllergyTags: () => null,
                    NutritionalItem: () => null,
                    ItemInfo: () => null,
                    ProviderInfo: (x) => (x as ProviderInfo).provider,
                    IngredientInfo: () => null,
                    CartTally: () => null,
                    MoreButton: () => null,
                    NumberOfServings: () => null,
                    ProductId: () => null,
                    SaleDescription: () => null,
                    WggSaleCategory: (x) => {
                        const data = x as WggSaleCategory;
                        const id = data.providerInfo.provider + data.name + data.id;
                        
                        return id;
                    }
                },
                updates: {
                    Mutation: {
                        logout(result, _variables, cache, _info) {
                            cache.invalidate({ __typename: 'AuthContext', id: result.logout as unknown as number });
                        }
                    }
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
): Promise<{ store: OperationResultStore<Data, Variables>; item: OperationResultState<Data, Variables> }> {
    const result = queryStore(args);

    let resolver: (value: any) => void;
    let rejector: (reason: any) => void;

    const finalPromise: Promise<{
        store: OperationResultStore<Data, Variables>;
        item: OperationResultState<Data, Variables>;
    }> = new Promise((accept, reject) => {
        resolver = accept;
        rejector = reject;
    });

    const unsubscribe = result.subscribe((x) => {
        if (x.data != undefined) {
            resolver({ store: result, item: x });
            // Hacky way to get around the situation where `x` has data immediately available.
            // The `unsubscribe()` method wouldn't be initialised yet, causing an error.

            setTimeout(() => unsubscribe(), 1);
        }

        if (!x.fetching) {
            if (x.error) {
                rejector(result);
                // Hacky way to get around the situation where `x` has data immediately available.
                // The `unsubscribe()` method wouldn't be initialised yet, causing an error.
                setTimeout(() => unsubscribe(), 1);
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
    args: MutationArgs<Data, Variables>,
    handler?: SubscriptionHandler<Data, Data>
): Promise<{ store: OperationResultStore<Data, Variables>; item: OperationResultState<Data, Variables> }> {
    const result = mutationStore(args, handler);

    let resolver: (value: any) => void;
    let rejector: (reason: any) => void;

    const finalPromise: Promise<{
        store: OperationResultStore<Data, Variables>;
        item: OperationResultState<Data, Variables>;
    }> = new Promise((accept, reject) => {
        resolver = accept;
        rejector = reject;
    });

    const unsubscribe = result.subscribe(async (x) => {
        if (x.data != undefined) {
            resolver({ store: result, item: x });
            // Hacky way to get around the situation where `x` has data immediately available.
            // The `unsubscribe()` method wouldn't be initialised yet, causing an error.
            setTimeout(() => unsubscribe(), 1);
        }

        if (!x.fetching) {
            if (x.error) {
                rejector(result);
                // Hacky way to get around the situation where `x` has data immediately available.
                // The `unsubscribe()` method wouldn't be initialised yet, causing an error.
                setTimeout(() => unsubscribe(), 1);
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
function _numberOrArray(ids: any): number[] {
    if (Array.isArray(ids)) {
        return ids;
    } else {
        return [ids];
    }
}

import {
    AggregateProductInput,
    CartCurrentQueryDocument,
    CartFragment,
    NoteProductInput,
    Provider,
    RawProductInput,
    RemoveProductFromCartDocument,
    RemoveRawProductInput,
    SetProductToCartDocument
} from '$lib/api/graphql_types';
import { asyncMutationStore, Client, queryStore } from '$lib/api/urql';
import { derived, Readable } from 'svelte/store';

export type ProductId = string;
export type CartStore = Readable<CartData | undefined>;

export interface CartData extends CartFragment {
    /**
     * Returns the quantity of this product that is present in this cart.
     */
    getProductQuantity(provider: Provider, productId: ProductId): QuantityInfo[];
}

/**
 * Initialise the global cart store, returns it to be passed to $page.data
 */
export function initialiseCart(client: Client): CartStore {
    const store = queryStore({ query: CartCurrentQueryDocument, client });

    return derived(store, (x, set) => {
        if (!x.fetching) {
            if (x.data) {
                const cartInfo = x.data.cartCurrent;

                set({
                    getProductQuantity: (provider, productId) => getProductQuantityImpl(cartInfo, provider, productId),
                    ...cartInfo
                });
            } else {
                set(undefined);
            }
        }
    });
}

/**
 * Set (or delete if `newQuantity == 0`) the provided product into the cart.
 */
export async function setCartContent(
    productInput: RawProductInput & { __typename: 'RawProduct' } | AggregateProductInput & { __typename: 'Aggregate' } | NoteProductInput & { __typename: 'Note' },
    client: Client
) {
    if (productInput.quantity != 0) {
        // We should set the item quantity.
        const input = {
            rawProduct:  productInput.__typename == "RawProduct" ? {
                productId: productInput.productId,
                provider: productInput.provider,
                quantity: productInput.quantity
            } : undefined,
            aggregate: productInput.__typename == "Aggregate" ? {
                aggregateId: productInput.aggregateId,
                quantity: productInput.quantity,
            } : undefined,
            notes: productInput.__typename == "Note" ? {
                id: productInput.id,
                content: productInput.content,
                quantity: productInput.quantity
            } : undefined
        };

        const _ = await asyncMutationStore({
            query: SetProductToCartDocument,
            variables: {
                input
            },
            client
        });
    } else {
        // We should remove the item
        const input = {
            rawProduct: productInput.__typename == "RawProduct" ? {
                productId: productInput.productId,
                provider: productInput.provider
            } : undefined,
            aggregate: productInput.__typename == "Aggregate" ? productInput.aggregateId : undefined,
            notes: productInput.__typename == "Note" && productInput.id ? productInput.id : undefined
        };
    
        const _ = await asyncMutationStore({
            query: RemoveProductFromCartDocument,
            variables: {
                input
            },
            client
        });
    }
}

export interface QuantityInfo {
    quantity: number;
    origin: 'Indirect' | 'Direct';
}

function getProductQuantityImpl(cart: CartFragment, provider: Provider, productId: ProductId): QuantityInfo[] {
    const results: QuantityInfo[] = cart.contents
        .filter((x) => x.__typename != 'CartNoteProduct')
        .map((x) => {
            if (x.__typename == 'CartAggregateProduct') {
                // Have to search constituent ingredients.
                const item = x.aggregate.ingredients.find(
                    (y) => y.id == productId && y.providerInfo.provider == provider
                );

                if (item != undefined) {
                    return {
                        quantity: x.quantity,
                        origin: 'Indirect'
                    };
                }
            } else if (x.__typename == 'CartProviderProduct') {
                if (x.product.id == productId && x.product.providerInfo.provider == provider) {
                    return {
                        quantity: x.quantity,
                        origin: 'Direct'
                    };
                }
            }

            return undefined;
        })
        .filter((x): x is QuantityInfo => x != undefined);

    return results;
}

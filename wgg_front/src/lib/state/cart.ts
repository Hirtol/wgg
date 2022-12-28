import {
    AggregateProductInput,
    CartCurrentQueryDocument,
    CartFragment,
    NoteProductInput,
    PriceFilter,
    Provider,
    RawProductInput,
    RemoveProductFromCartDocument,
    SetProductToCartDocument
} from '$lib/api/graphql_types';
import { asyncMutationStore, Client } from '$lib/api/urql';
import { derived, get, Readable, writable } from 'svelte/store';
import { PreferenceStore } from './preferences';

export type ProductId = string;

export interface CartStore extends Readable<CartData> {
    /**
     * Set (or delete if `newQuantity == 0`) the provided product into the cart.
     */
    setCartContent(
        productInput:
            | (RawProductInput & { __typename: 'RawProduct' })
            | (AggregateProductInput & { __typename: 'Aggregate' })
            | (NoteProductInput & { __typename: 'Note' }),
        client: Client
    ): Promise<void>;

    /**
     * Refreshes the current cart's contents
     */
    refreshContent(client: Client): Promise<void>;
}

export interface CartData {
    /**
     * Returns the quantity of this product that is present in this cart.
     */
    getProductQuantity(provider: Provider, productId: ProductId): QuantityInfo[];

    /**
     * Returns the quantity of this aggregate product that is present in this cart.
     */
    getAggregateQuantity(aggregateId: number): QuantityInfo | undefined;

    data: CartFragment | undefined;
}

export async function initialiseRealCart(
    client: Client,
    pricePreference: PreferenceStore,
    cartData?: CartFragment
): Promise<CartStore> {
    let prefs = get(pricePreference);
    const cartInfo = cartData
        ? cartData
        : (await client.query(CartCurrentQueryDocument, { price: prefs.aggregateDisplayPrice }).toPromise()).data
              ?.cartCurrent;

    const backingStore = writable(createCartData(cartInfo));
    const { set } = backingStore;
    const resultStore: Readable<CartData> = derived([backingStore, pricePreference], (values, set_derived) => {
        prefs = values[1];
        set_derived(values[0]);
    });

    return {
        subscribe: resultStore.subscribe,
        setCartContent: async (productInput, client) => {
            const data = await setCartContent(productInput, prefs.aggregateDisplayPrice, client);

            if (data) {
                set(createCartData(data));
            }
        },
        refreshContent: async () => {
            const item = await client
                .query(
                    CartCurrentQueryDocument,
                    { price: prefs.aggregateDisplayPrice },
                    { requestPolicy: 'cache-and-network' }
                )
                .toPromise();

            if (item.data) {
                set(createCartData(item.data.cartCurrent));
            }
        }
    };
}

function createCartData(cart: CartFragment | undefined): CartData {
    return {
        getProductQuantity: (provider: Provider, productId: string) =>
            cart ? getProductQuantityImpl(cart, provider, productId) : [],
        getAggregateQuantity: (aggregateId) => (cart ? getAggregateQuantityImpl(cart, aggregateId) : undefined),
        data: cart
    };
}

/**
 * Set (or delete if `newQuantity == 0`) the provided product into the cart.
 */
async function setCartContent(
    productInput:
        | (RawProductInput & { __typename: 'RawProduct' })
        | (AggregateProductInput & { __typename: 'Aggregate' })
        | (NoteProductInput & { __typename: 'Note' }),
    priceFilter: PriceFilter,
    client: Client
): Promise<CartFragment | undefined> {
    if (productInput.quantity != 0) {
        // We should set the item quantity.
        const input = {
            rawProduct:
                productInput.__typename == 'RawProduct'
                    ? {
                          productId: productInput.productId,
                          provider: productInput.provider,
                          quantity: productInput.quantity
                      }
                    : undefined,
            aggregate:
                productInput.__typename == 'Aggregate'
                    ? {
                          aggregateId: productInput.aggregateId,
                          quantity: productInput.quantity
                      }
                    : undefined,
            notes:
                productInput.__typename == 'Note'
                    ? {
                          id: productInput.id,
                          content: productInput.content,
                          quantity: productInput.quantity
                      }
                    : undefined
        };

        const { item } = await asyncMutationStore(
            {
                query: SetProductToCartDocument,
                variables: {
                    input,
                    price: priceFilter
                },
                client
            },
            undefined,
            false
        );

        return item.data?.cartCurrentSetProduct.data;
    } else {
        // We should remove the item
        const input = {
            rawProduct:
                productInput.__typename == 'RawProduct'
                    ? {
                          productId: productInput.productId,
                          provider: productInput.provider
                      }
                    : undefined,
            aggregate: productInput.__typename == 'Aggregate' ? productInput.aggregateId : undefined,
            notes: productInput.__typename == 'Note' && productInput.id ? productInput.id : undefined
        };

        const { item } = await asyncMutationStore(
            {
                query: RemoveProductFromCartDocument,
                variables: {
                    input,
                    price: priceFilter
                },
                client
            },
            undefined,
            false
        );

        return item.data?.cartCurrentRemoveProduct.data;
    }
}

export interface QuantityInfo {
    quantity: number;
    origin: 'Indirect' | 'Direct';
}

function getAggregateQuantityImpl(cart: CartFragment, aggregateId: number): QuantityInfo | undefined {
    const item = cart.contents.find((x) => x.__typename == 'CartAggregateProduct' && x.aggregate.id == aggregateId);
    return item ? { quantity: item.quantity, origin: 'Direct' } : undefined;
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

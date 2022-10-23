<script lang="ts">
    import {
        CartContentFragment,
        ProductCardFragment,
        Provider,
        RemoveProductFromCartDocument,
        SetProductToCartDocument
    } from '$lib/api/graphql_types';
    import { asyncMutationStore, getContextClient } from '$lib/api/urql';
    import AddComponent from '$lib/components/products/AddComponent.svelte';
    import PriceComponent from '$lib/components/products/PriceComponent.svelte';
    import ProductImage from '$lib/components/products/ProductImage.svelte';
    import { setCartContent } from '$lib/state';
    import { centsToPrice, centsToTextPrice, unitToText } from '$lib/utils';
    import { tooltip } from '@brainandbones/skeleton';
    import { Information, Pen } from 'carbon-icons-svelte';
    import classNames from 'classnames';

    const client = getContextClient();

    export { className as class };
    export let data: CartContentFragment;

    let className: string;
    let quantity: number = data.quantity;

    $: classes = classNames('card !bg-surface-50 dark:!bg-surface-700/75 h-full flex flex-row', className);
    $: productUrl = getProductUrl(data);
    $: imageData = getImageData(data);
    $: cardTitle = getCardTitle(data);
    $: priceData = getPriceData(data);

    $: saleLabel =
        data.__typename == 'CartProviderProduct' && data.product.decorators.find((l) => l.__typename == 'SaleLabel');
    $: unavailableReason =
        data.__typename == 'CartProviderProduct'
            ? data.product.decorators.find((u) => u.__typename == 'UnavailableItem')
            : undefined;

    function getProductUrl(data: CartContentFragment): string | undefined {
        if (data.__typename == 'CartAggregateProduct') {
            return `/aggregates/${data.aggregate.id}`;
        } else if (data.__typename == 'CartProviderProduct') {
            return `/products/${data.product.providerInfo.provider}/${data.product.id}`;
        } else if (data.__typename == 'CartNoteProduct') {
            return undefined;
        }
    }

    function getImageData(data: CartContentFragment): { imageUrl?: string; name: string } | undefined {
        if (data.__typename == 'CartAggregateProduct') {
            return { imageUrl: data.aggregate.imageUrl, name: data.aggregate.name };
        } else if (data.__typename == 'CartProviderProduct') {
            return { imageUrl: data.product.imageUrl, name: data.product.name };
        } else if (data.__typename == 'CartNoteProduct') {
            return undefined;
        }
    }

    function getCardTitle(data: CartContentFragment): string | undefined {
        if (data.__typename == 'CartAggregateProduct') {
            return data.aggregate.name;
        } else if (data.__typename == 'CartProviderProduct') {
            return data.product.name;
        } else if (data.__typename == 'CartNoteProduct') {
            return data.note;
        }
    }

    function getPriceData(data: CartContentFragment): { displayPrice: number; fullPrice: number } | undefined {
        if (data.__typename == 'CartAggregateProduct') {
            return { displayPrice: data.aggregate.price, fullPrice: data.aggregate.price };
        } else if (data.__typename == 'CartProviderProduct') {
            return data.product;
        } else if (data.__typename == 'CartNoteProduct') {
            return undefined;
        }
    }

    async function updateCartContent(newQuantity: number) {
        quantity = newQuantity;
        if (data.__typename == 'CartAggregateProduct') {
            await setCartContent(
                {
                    aggregateId: data.aggregate.id,
                    quantity,
                    __typename: 'Aggregate'
                },
                client
            );
        } else if (data.__typename == 'CartProviderProduct') {
            await setCartContent(
                {
                    productId: data.product.id,
                    provider: data.product.providerInfo.provider,
                    quantity,
                    __typename: 'RawProduct'
                },
                client
            );
        } else if (data.__typename == 'CartNoteProduct') {
            await setCartContent(
                {
                    id: data.id,
                    content: data.note,
                    quantity,
                    __typename: 'Note'
                },
                client
            );
        }
    }
</script>

<div class="card flex h-full flex-row overflow-hidden !bg-surface-50 p-2 transition-all dark:!bg-surface-700/75">
    <!-- Left -->
    <div class="flex flex-row items-center justify-start gap-2">
        <AddComponent quantity={data.quantity} on:setQuantity={async (e) => await updateCartContent(e.detail)} />
        <!-- Images -->
        <div class="flex h-16 min-w-[4rem] content-center items-center">
            {#if imageData != undefined}
                <!-- Normal Images -->
                <ProductImage data={imageData} class="!h-16 !w-16" blurImage={unavailableReason != undefined} />
            {:else}
                <!-- Notes -->
                <Pen class="h-[80%] w-full" />
            {/if}
        </div>

        <!-- Title/Sale/Unavailable -->
        <svelte:element this={productUrl != undefined ? 'a' : 'div'} class="unstyled flex flex-col" href={productUrl}>
            <h6 class="line-clamp-2">{cardTitle}</h6>

            {#if saleLabel && saleLabel.__typename == 'SaleLabel'}
                <span class="badge w-min bg-primary-300 dark:bg-primary-800">{saleLabel.text}</span>
            {/if}

            {#if unavailableReason != undefined && unavailableReason.__typename == 'UnavailableItem'}
                <h6 class="text-xs text-black/70 line-clamp-2 dark:text-white/70">
                    {unavailableReason.explanationShort}
                </h6>
            {/if}
        </svelte:element>
    </div>

    <!-- Right -->
    <div class="ml-auto flex flex-row">
        {#if priceData != undefined}
            <PriceComponent
                dashed={unavailableReason != undefined}
                data={priceData}
                class="mt-auto whitespace-nowrap" />
        {/if}
    </div>
</div>

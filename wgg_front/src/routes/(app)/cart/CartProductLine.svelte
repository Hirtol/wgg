<script lang="ts">
    import { CartContentFragment } from '$lib/api/graphql_types';
    import { getContextClient } from '$lib/api/urql';
    import AddComponent from '$lib/components/product_display/products/AddComponent.svelte';
    import PriceComponent from '$lib/components/product_display/products/PriceComponent.svelte';
    import ProductImage from '$lib/components/product_display/products/ProductImage.svelte';
    import SaleLabel from '$lib/components/product_display/products/SaleLabel.svelte';
    import { CartStore } from '$lib/state';
    import { Pen } from 'carbon-icons-svelte';
    import classNames from 'classnames';

    const client = getContextClient();

    export { className as class };
    export let data: CartContentFragment;
    export let cart: CartStore;

    let className: string = '';
    let quantity: number = data.quantity;

    $: classes = classNames(
        'card flex flex-row justify-between overflow-hidden !bg-surface-50 p-2 transition-all dark:!bg-surface-700/75',
        className
    );
    $: productUrl = getProductUrl(data);
    $: imageData = getImageData(data);
    $: cardTitle = getCardTitle(data);
    $: priceData = getPriceData(data);

    $: saleInfo = data.__typename == 'CartProviderProduct' && data.product.saleInformation;
    $: unavailableReason = data.__typename == 'CartProviderProduct' ? data.product.unavailableDetails : undefined;

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

    function getPriceData(data: CartContentFragment): { displayPrice: number; originalPrice: number } | undefined {
        if (data.__typename == 'CartAggregateProduct') {
            return { displayPrice: data.aggregate.price, originalPrice: data.aggregate.price };
        } else if (data.__typename == 'CartProviderProduct') {
            return data.product.priceInfo;
        } else if (data.__typename == 'CartNoteProduct') {
            return undefined;
        }
    }

    async function updateCartContent(newQuantity: number) {
        quantity = newQuantity;
        if (data.__typename == 'CartAggregateProduct') {
            await cart.setCartContent(
                {
                    aggregateId: data.aggregate.id,
                    quantity,
                    __typename: 'Aggregate'
                },
                client
            );
        } else if (data.__typename == 'CartProviderProduct') {
            await cart.setCartContent(
                {
                    productId: data.product.id,
                    provider: data.product.providerInfo.provider,
                    quantity,
                    __typename: 'RawProduct'
                },
                client
            );
        } else if (data.__typename == 'CartNoteProduct') {
            await cart.setCartContent(
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

<div class={classes}>
    <!-- Left -->
    <div class="flex min-w-0 flex-row items-center justify-start gap-2">
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
        <svelte:element
            this={productUrl != undefined ? 'a' : 'div'}
            class="unstyled flex min-w-0 flex-col"
            href={productUrl}>
            <h6 class="line-clamp-2">{cardTitle}</h6>

            {#if saleInfo}
                <SaleLabel class="min-w-0" text={saleInfo.label} saleType={saleInfo.saleType} />
            {/if}

            {#if unavailableReason}
                <h6 class="text-xs text-black/70 line-clamp-2 dark:text-white/70">
                    {unavailableReason.explanationShort}
                </h6>
            {/if}
        </svelte:element>
    </div>

    <!-- Right -->
    <div class="flex flex-col relative">
        <!-- Raw product provider indicator -->
        {#if data.__typename == 'CartProviderProduct'}
            <img
                src={data.product.providerInfo.logoUrl}
                class="pointer-events-none w-8 absolute top-0 right-0"
                alt={data.product.providerInfo.provider} />
        {/if}

        {#if priceData != undefined}
            <PriceComponent
                dashed={unavailableReason != undefined}
                data={priceData}
                class="mt-auto whitespace-nowrap" />
        {/if}
    </div>
</div>

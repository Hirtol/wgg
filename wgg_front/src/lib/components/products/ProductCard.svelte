<script lang="ts">
    import {
        ProductCardFragment,
        Provider,
        RemoveProductFromCartDocument,
        SetProductToCartDocument
    } from '$lib/api/graphql_types';
    import { asyncMutationStore, getContextClient } from '$lib/api/urql';
    import { setCartContent } from '$lib/state';
    import { centsToPrice, centsToTextPrice, unitToText } from '$lib/utils';
    import { tooltip } from '@brainandbones/skeleton';
    import { Information } from 'carbon-icons-svelte';
    import classNames from 'classnames';
    import { notifications } from '../notifications/notification';
    import AddComponent from './AddComponent.svelte';
    import PriceComponent from './PriceComponent.svelte';
    import ProductImage from './ProductImage.svelte';

    export let data: ProductCardFragment;

    export let quantity: number;

    const client = getContextClient();

    $: classes = classNames(
        $$restProps.class,
        'card card-body !bg-surface-50 dark:!bg-surface-700/75 h-full flex flex-col'
    );
    $: saleLabel = data.decorators.find((l) => l.__typename == 'SaleLabel');

    $: productUrl = `/products/${data.providerInfo.provider}/${data.id}`;

    $: unavailableReason = data.decorators.find((u) => u.__typename == 'UnavailableItem');

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        quantity = newQuantity;
        await setCartContent({productId, provider, quantity, __typename: "RawProduct"}, client);
    }
</script>

<div class={classes}>
    <header class="relative mx-auto">
        <ProductImage {data} blurImage={!data.available} />

        {#if !data.available && unavailableReason?.__typename == 'UnavailableItem'}
            <p class="absolute bottom-0 left-0 w-full !text-warning-600 line-clamp-2 dark:!text-warning-300">
                {unavailableReason.explanationShort}
            </p>
        {/if}

        <img
            src={data.providerInfo.logoUrl}
            class="pointer-events-none absolute top-0 right-0 w-1/6"
            alt={data.providerInfo.provider} />
    </header>

    <div class="body">
        <a class="unstyled text-base font-bold line-clamp-2 md:text-xl" title={data.name} href={productUrl}
            >{data.name}</a>

        <div class="text-s flex text-gray-500 dark:text-gray-400">
            <h6 class="text-s flex text-gray-500 line-clamp-1 dark:text-gray-400">
                {data.unitQuantity.amount}
                {unitToText(data.unitQuantity.unit, true, false)}

                {#if data.unitPrice}
                    €{centsToTextPrice(data.unitPrice.price)}/{unitToText(data.unitPrice.unit, true, false)}
                {/if}
            </h6>

            <AddComponent
                class="ml-auto inline-block !h-6"
                {quantity}
                on:setQuantity={(e) => updateCartContent(data.id, data.providerInfo.provider, e.detail)} />
        </div>

        {#if saleLabel && saleLabel.__typename == 'SaleLabel'}
            <span class="badge bg-primary-300 dark:bg-primary-800">{saleLabel.text}</span>
        {/if}
    </div>

    <div class="footer mt-auto justify-end pt-1">
        <div class="relative flex flex-row items-center justify-between">
            <a href={productUrl} title={data.name} class="unstyled">
                <Information />
            </a>

            <PriceComponent {data} />
        </div>
    </div>
</div>

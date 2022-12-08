<!-- 
    @component
    Displays the given product as a friendly card.

    Will publish any changes to its own quantity as a `updateCartContent` event.
    The caller is expected to deal with this.
 -->
<script lang="ts">
    import { ProductCardFragment, Provider } from '$lib/api/graphql_types';
    import { productPageItemUrl } from '$lib/routing';
    import { centsToTextPrice, unitToText } from '$lib/utils';
    import { Information, Warning } from 'carbon-icons-svelte';
    import { createEventDispatcher } from 'svelte';
    import AddComponent from './AddComponent.svelte';
    import PriceComponent from './PriceComponent.svelte';
    import PriceQuantityComponent from './PriceQuantityComponent.svelte';
    import ProductCardSkeleton from './ProductCardSkeleton.svelte';
    import ProductImage from './ProductImage.svelte';
    import SaleLabel from './SaleLabel.svelte';

    export { className as class };

    export let data: ProductCardFragment;

    export let quantity: number;

    const dispatch = createEventDispatcher<{
        updateCartContent: { productId: string; provider: Provider; newQuantity: number };
    }>();

    let className: string = '';

    $: saleInfo = data.saleInformation;

    $: productUrl = productPageItemUrl(data.providerInfo.provider, data.id);

    $: unavailableReason = data.decorators.find((u) => u.__typename == 'UnavailableItem');

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        quantity = newQuantity;
        dispatch('updateCartContent', { productId, provider, newQuantity });
    }
</script>

<ProductCardSkeleton class={className}>
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

        <!-- Quantity and Add/Remove -->
        <div class="text-s flex text-gray-500 dark:text-gray-400">
            <PriceQuantityComponent
                data={{ unitQuantity: data.unitQuantity, priceInfo: { unitPrice: data.unitPrice } }} />

            <AddComponent
                class="ml-auto inline-block !h-6"
                {quantity}
                on:setQuantity={(e) => updateCartContent(data.id, data.providerInfo.provider, e.detail)} />
        </div>

        <!-- Sale Label -->
        {#if saleInfo}
            <SaleLabel text={saleInfo.label} saleType={saleInfo.saleType} />
        {/if}
    </div>

    <!-- Footer/Price -->
    <div class="footer mt-auto justify-end pt-1">
        <div class="relative flex flex-row items-center justify-between">
            <a href={productUrl} title={data.name} class="unstyled">
                <Information />
            </a>

            <PriceComponent {data} />
        </div>
    </div>
</ProductCardSkeleton>

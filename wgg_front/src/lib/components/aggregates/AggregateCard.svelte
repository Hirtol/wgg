<!--
    @component
    A partial Product card dedicated for displaying an aggregate ingredient
-->
<script lang="ts">
    import { AggregateFragment, Provider } from '$lib/api/graphql_types';
    import ProductCardSkeleton from '$lib/components/products/ProductCardSkeleton.svelte';
    import ProductImage from '$lib/components/products/ProductImage.svelte';
    import SaleLabel from '$lib/components/products/SaleLabel.svelte';
    import { aggregatePageItemUrl, salesPageItemUrl } from '$lib/routing';

    export let data: AggregateFragment;

    $: listUrl = aggregatePageItemUrl(data.id);
</script>

<ProductCardSkeleton>
    <header class="relative mx-auto">
        <ProductImage data={{ name: data.name, imageUrl: data.imageUrl }} />
    </header>

    <div class="body">
        <a class="unstyled text-base font-bold line-clamp-2 md:text-xl" title={data.name} href={listUrl}>{data.name}</a>

        <!-- Quantity and Add/Remove -->
        <div class="text-s flex text-gray-500 dark:text-gray-400">
            <!-- <PriceQuantityComponent
                data={{ unitQuantity: data.unitQuantity, priceInfo: data.priceInfo }} />

            <AddComponent
                class="ml-auto inline-block !h-6"
                {quantity}
                on:setQuantity={(e) => updateCartContent(data.id, data.providerInfo.provider, e.detail)} /> -->
        </div>
    </div>

    <!-- <div class="footer flex h-full flex-col justify-end pt-1">
        {#if data.saleDescription}
            <p class="mt-auto line-clamp-2">{@html data.saleDescription}</p>
        {/if}
    </div> -->
</ProductCardSkeleton>

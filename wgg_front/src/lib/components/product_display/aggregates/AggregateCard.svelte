<!--
    @component
    A partial Product card dedicated for displaying an aggregate ingredient
-->
<script lang="ts">
    import {AggregateCardFragment} from '$lib/api/graphql_types';
    import ProductCardSkeleton from '$lib/components/product_display/products/ProductCardSkeleton.svelte';
    import ProductImage from '$lib/components/product_display/products/ProductImage.svelte';
    import {aggregatePageItemUrl} from '$lib/routing';
    import { Information } from 'carbon-icons-svelte';
    import { createEventDispatcher } from 'svelte';
    import AddComponent from '../products/AddComponent.svelte';
    import PriceComponent from '../products/PriceComponent.svelte';
    import SaleLabel from '../products/SaleLabel.svelte';

    const dispatch = createEventDispatcher<{
        updateCartContent: { aggregateId: number; newQuantity: number };
    }>();

    export { _class as class };

    export let data: AggregateCardFragment;

    export let quantity: number;

    let _class: string = '';

    async function updateCartContent(aggregateId: number, newQuantity: number) {
        dispatch('updateCartContent', { aggregateId, newQuantity });
    }

    $: listUrl = aggregatePageItemUrl(data.id);
    $: saleInfo = data.sales
</script>

<ProductCardSkeleton class={_class}>
    <header class="relative mx-auto">
        <ProductImage data={{ name: data.name, imageUrl: data.imageUrl }}/>

        <!-- Sale Label -->
        {#if saleInfo.length > 0}
            {#each saleInfo as sale}
                <SaleLabel class="absolute bottom-0 left-0 w-full" text={sale.label} saleType={sale.saleType} />
            {/each}
        {/if}
    </header>

    <div class="body">
        <a class="unstyled text-base font-bold line-clamp-2 md:text-xl" title={data.name} href={listUrl}>{data.name}</a>

        <!-- Quantity and Add/Remove -->
        <div class="text-s flex text-gray-500 dark:text-gray-400">

            <AddComponent
                class="ml-auto inline-block !h-6"
                {quantity}
                on:setQuantity={(e) => updateCartContent(data.id, e.detail)} />
        </div>
    </div>

    <!-- Footer/Price -->
    <div class="footer mt-auto justify-end pt-1">
        <div class="relative flex flex-row items-center justify-between">
            <a href={listUrl} title={data.name} class="unstyled">
                <Information />
            </a>

            <PriceComponent data={{displayPrice: data.price, originalPrice: data.price}} />
        </div>
    </div>
</ProductCardSkeleton>

<!--
    @component
    A heterogeneous list for card display, this accepts both SaleGroupCards and ProductCards
-->
<script context="module" lang="ts">
    import { AggregateCardFragment, ProductCardFragment, SaleGroupFragment } from '$lib/api/graphql_types';

    export type HeteroCardData = ProductCardFragment | AggregateCardFragment | SaleGroupFragment;
</script>

<script lang="ts">
    import ProductCard from '$lib/components/product_display/products/ProductCard.svelte';
    import CardListSkeleton from '$lib/components/product_display/CardListSkeleton.svelte';
    import { CartStore } from '$lib/state';
    import SaleGroupCard from './sales/SaleGroupCard.svelte';
    import AggregateCard from '$lib/components/product_display/aggregates/AggregateCard.svelte';

    export let data: HeteroCardData[];
    export let cartStore: CartStore;

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];
</script>

<CardListSkeleton {cartStore} {columns}>
    <svelte:fragment let:updateCartContent>
        {#each data as card (card.id + (card.__typename ?? ''))}
            {#if card.__typename === 'WggSearchProduct'}
                <ProductCard
                    data={card}
                    on:updateCartContent={updateCartContent}
                    quantity={$cartStore.getProductQuantity(card.providerInfo.provider, card.id)[0]?.quantity ?? 0} />
            {:else if card.__typename === 'WggSaleGroupLimited'}
                <SaleGroupCard data={card} />
            {:else if card.__typename === 'AggregateIngredient'}
                <AggregateCard data={card} />
            {/if}
        {/each}

        <slot name="last-grid" />
    </svelte:fragment>
</CardListSkeleton>

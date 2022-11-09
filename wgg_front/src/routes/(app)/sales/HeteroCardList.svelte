<!--
    @component
    A heterogeneous list for card display, this accepts both SaleGroupCards and ProductCards
-->
<script lang="ts">
    import { SaleCategoryFragment } from '$lib/api/graphql_types';
    import ProductCard from '$lib/components/products/ProductCard.svelte';
    import ProductListSkeleton from '$lib/components/products/ProductListSkeleton.svelte';
    import { CartStore } from '$lib/state';
    import SaleGroupCard from './SaleGroupCard.svelte';

    export let data: SaleCategoryFragment;
    export let cartStore: CartStore;

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];
</script>

<ProductListSkeleton {cartStore} {columns}>
    <svelte:fragment let:updateCartContent>
        {#each data.items as card (card.id)}
            {#if card.__typename == 'WggSearchProduct'}
                <ProductCard
                    data={card}
                    on:updateCartContent={updateCartContent}
                    quantity={$cartStore.getProductQuantity(card.providerInfo.provider, card.id)[0]?.quantity ?? 0}
                    class="w-full" />
            {:else if card.__typename == 'WggSaleGroupLimited'}
                <SaleGroupCard data={card} provider={data.providerInfo.provider} />
            {/if}
        {/each}

        <slot name="last-grid" />
    </svelte:fragment>
</ProductListSkeleton>

<!-- 
    @component
    Displays the given product cards in a grid following the rules established in `columns`.

    Automatically handles `ProductCard` update cart content requests.
 -->
<script lang="ts">
    import { ProductCardFragment, Provider } from '$lib/api/graphql_types';
    import { CartStore } from '$lib/state';
    import ProductCard from './ProductCard.svelte';
    import ProductListSkeleton from './ProductListSkeleton.svelte';

    export let data: ProductCardFragment[];
    export let cartStore: CartStore;

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];
</script>

<ProductListSkeleton {cartStore} {columns}>
    <svelte:fragment let:updateCartContent>
        {#each data as card (card.id)}
            <ProductCard
                data={card}
                on:updateCartContent={updateCartContent}
                quantity={$cartStore.getProductQuantity(card.providerInfo.provider, card.id)[0]?.quantity ?? 0}
                class="w-full" />
        {/each}

        <slot name="last-grid" />
    </svelte:fragment>
</ProductListSkeleton>

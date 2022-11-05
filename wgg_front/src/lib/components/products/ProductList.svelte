<!-- 
    @component
    Displays the given product cards in a grid following the rules established in `columns`.

    Automatically handles `ProductCard` update cart content requests.
 -->
<script lang="ts">
    import { ProductCardFragment, Provider } from '$lib/api/graphql_types';
    import { getContextClient } from '$lib/api/urql';
    import { CartStore } from '$lib/state';
    import ProductCard from './ProductCard.svelte';

    export let data: ProductCardFragment[];
    export let cartStore: CartStore;

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];

    const client = getContextClient();

    async function updateCartContent(
        event: CustomEvent<{
            productId: string;
            provider: Provider;
            newQuantity: number;
        }>
    ) {
        let { productId, provider, newQuantity } = event.detail;
        await cartStore.setCartContent(
            { productId, provider, quantity: newQuantity, __typename: 'RawProduct' },
            client
        );
    }
</script>

<div class="grid gap-0.5 md:gap-2 {columns.join(' ')}">
    {#each data as card (card.id)}
        <ProductCard
            data={card}
            on:updateCartContent={updateCartContent}
            quantity={$cartStore.getProductQuantity(card.providerInfo.provider, card.id)[0]?.quantity ?? 0}
            class="w-full" />
    {/each}

    <slot name="last-grid" />
</div>

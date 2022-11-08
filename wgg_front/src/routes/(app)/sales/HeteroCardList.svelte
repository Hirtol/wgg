<!--
    @component
    A heteregenous list for card display, this accepts both SaleGroupCards and ProductCards
-->
<script lang="ts">
    import { ProductCardFragment, Provider, SaleCategoryFragment } from '$lib/api/graphql_types';
    import { getContextClient } from '$lib/api/urql';
    import ProductCard from '$lib/components/products/ProductCard.svelte';
    import { CartStore } from '$lib/state';
    import SaleGroupCard from './SaleGroupCard.svelte';

    export let data: SaleCategoryFragment;
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
    {#each data.items as card (card.id)}
        {#if card.__typename == 'WggSearchProduct'}
            <ProductCard
                data={card}
                on:updateCartContent={updateCartContent}
                quantity={$cartStore.getProductQuantity(card.providerInfo.provider, card.id)[0]?.quantity ?? 0}
                class="w-full" />
        {:else if card.__typename == 'WggSaleGroupLimited'}
            <SaleGroupCard data={card} provider={data.providerInfo.provider}></SaleGroupCard>
        {/if}
    {/each}
    <slot name="last-grid" />
</div>

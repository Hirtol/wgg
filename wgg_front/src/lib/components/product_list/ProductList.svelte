<script lang="ts">
    import { ProductCardFragment } from '$lib/api/graphql_types';
    import ProductCard from './ProductCard.svelte';

    export let data: [ProductCardFragment];

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];

    $: console.log(data.map(i => i.id).sort());
</script>

<div class="grid gap-0.5 md:gap-4 {columns.join(' ')}">
    {#each data as card (card.id)}
        <ProductCard data={card} class="w-full" />
    {/each}
</div>

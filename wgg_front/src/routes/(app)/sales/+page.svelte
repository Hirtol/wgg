<script lang="ts">
    import PageRoot from '$lib/components/PageRoot.svelte';
    import ProductList from '$lib/components/products/ProductList.svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ result, cart, preferences } = data);
</script>

<PageRoot>
    <p>Sales Page</p>
    <!-- <ProductList cart={$cart} data={$result} cartStore={cart} /> -->
</PageRoot>

<script lang="ts">
    import PageRoot from '$lib/components/PageRoot.svelte';
    import ProductList from '$lib/components/products/ProductList.svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ listStore, cart } = data);
</script>

<PageRoot>
    <ProductList cartStore={cart} data={$listStore.data?.proPromotionsSublist.items ?? []} />
</PageRoot>

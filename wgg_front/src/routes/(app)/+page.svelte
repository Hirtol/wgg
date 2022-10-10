<script lang="ts">
    import { getContextClient } from '$lib/api/urql';
    import AddComponent from '$lib/components/product_list/AddComponent.svelte';
    import ProductList from '$lib/components/product_list/ProductList.svelte';
    import { LightSwitch } from '@brainandbones/skeleton';
    import type { PageData } from './$types';

    export let data: PageData;

    const client = getContextClient();

    $: ({ result } = data);

    $: firstItem = $result.data?.proPromotions[0].limitedItems;
    $: searchItems = $result.data?.proSearchAll;
</script>

<LightSwitch />

<!-- <AppBar>
    <svelte:fragment slot="lead">
        <span class="text-base font-bold uppercase md:text-3xl">Logo</span>
    </svelte:fragment>
    <div class="flex flex-row">
        <input type="search" placeholder="Search..." class="mr-5 max-w-[200px] transition-all focus-within:max-w-lg" />
    </div>
    <svelte:fragment slot="trail">
        <div class="flex items-center space-x-6">
            <a href="/components/app-bar">Link</a>
        </div>
    </svelte:fragment>
</AppBar>

<p>Hellossss</p> -->
<a href="/login">awdawd</a>

<main class="container mx-auto px-0.5 md:px-0">
    <AddComponent normalButton permanentlyExpanded quantity={0} class="max-w-full" />
    <ProductList data={firstItem} />
    <!-- {#if firstItem}
            <ProductCard class="max-w-[15rem]" data={firstItem} />
        {/if} -->
    <ProductList data={searchItems} />
</main>

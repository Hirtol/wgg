<script lang="ts">
    // import { PageData } from '$types';
    import { AppBar, LightSwitch } from '@brainandbones/skeleton';
    import ProductCard from '$lib/components/product_list/ProductCard.svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ result } = data);

    $: firstItem = $result.data?.proSearchAll[0];
</script>

<LightSwitch />

<AppBar>
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

<p>Hellossss</p>
<a href="/login">awdawd</a>

{#if firstItem}
    <ProductCard data={firstItem} />
{/if}

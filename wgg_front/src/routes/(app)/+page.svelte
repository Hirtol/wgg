<script lang="ts">
    import { page } from '$app/stores';
    import { GetAllPromotionsDocument } from '$lib/api/graphql_types';
    import { asyncQueryStore, getContextClient } from '$lib/api/urql';
    import AddComponent from '$lib/components/products/AddComponent.svelte';
    import ProductList from '$lib/components/products/ProductList.svelte';
    import { LightSwitch } from '@brainandbones/skeleton';
    import type { PageData } from './$types';

    const client = getContextClient();

    export let data: PageData;

    let searchText: string;

    $: ({ result } = data);
    $: ({ cart } = $page.data);

    $: firstItem = $result.data?.proPromotions[0].limitedItems;
    $: searchItems = $result.data?.proSearchAll;

    async function newStuff(text: string) {
        data.result = (
            await asyncQueryStore({
                query: GetAllPromotionsDocument,
                client: client,
                variables: { search: text }
            })
        ).store;
    }
</script>

<svelte:head>
    <title>Wgg - Home</title>
</svelte:head>

<LightSwitch />

<form on:submit|preventDefault={async () => await newStuff(searchText)}>
    <input type="search" bind:value={searchText} placeholder="Product Text" />
    <button class="btn rounded bg-accent-600" on:click={async () => await newStuff(searchText)}>Search</button>
</form>

{#if $cart}
    <p>Cart Funds:</p>
    {#each $cart.tallies as tally (tally.providerInfo.provider)}
        <p>{tally.providerInfo.provider} - {tally.priceCents}</p>
    {/each}
{/if}

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
    {#if $cart && firstItem && searchItems}
        <ProductList cart={$cart} data={firstItem} />
        <!-- {#if firstItem}
                <ProductCard class="max-w-[15rem]" data={firstItem} />
            {/if} -->
        <ProductList cart={$cart} data={searchItems} />
    {/if}
</main>

<script lang="ts">
    import { page } from '$app/stores';
    import { GetAllPromotionsDocument } from '$lib/api/graphql_types';
    import { asyncQueryStore, getContextClient } from '$lib/api/urql';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import AddComponent from '$lib/components/product_display/products/AddComponent.svelte';
    import { LightSwitch } from '@skeletonlabs/skeleton';
    import type { PageData } from './$types';

    const client = getContextClient();

    export let data: PageData;

    let searchText: string;

    $: ({ result } = data);
    $: cart = data.cart;

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
    {#each $cart.data?.tallies ?? [] as tally (tally.providerInfo.provider)}
        <p>{tally.providerInfo.provider} - {tally.priceCents}</p>
    {/each}
{/if}

<main class="container mx-auto px-0.5 md:px-0">
    <AddComponent normalButton permanentlyExpanded quantity={0} class="max-w-full" />
    {#if $cart && searchItems}
        <HeteroCardList data={searchItems} cartStore={cart} />
    {/if}
</main>

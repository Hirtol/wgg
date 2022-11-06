<!--
    @component
    The page to display all sales. 
    We keep track of the current selected provider by using the query parameter 'provider' for ensuring that users examining a product get pushed back
    to the right sales page.
-->
<script lang="ts">
    import { afterNavigate, goto } from '$app/navigation';
    import { page } from '$app/stores';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import ProductList from '$lib/components/products/ProductList.svelte';
    import { getProviders } from '$lib/state';
    import { capitaliseFirst, updateQueryParameter } from '$lib/utils';
    import type { PageData } from './$types';

    export let data: PageData;

    let currentProvider = data.initialProvider;

    $: ({ result, cart, preferences } = data);

    // Navigate directly to the correct provider.
    updateQueryParameter($page.url, 'provider', data.initialProvider, { replaceState: true });

    afterNavigate(() => {
        // We want to update the currentProvider in case the Query parameter changed
        currentProvider = data.initialProvider;
    });

    async function updateHistory(_: Event) {
        await updateQueryParameter($page.url, 'provider', currentProvider, { replaceState: false });
    }
</script>

<svelte:head>
    <title>Wgg - Sales - {capitaliseFirst(currentProvider)}</title>
</svelte:head>

<PageRoot>
    <div class="grid grid-cols-1 gap-y-4 md:grid-cols-4 md:gap-4">
        <div id="product-list-section" class="order-2 md:order-1 md:col-span-3">
            <p>t</p>
        </div>
        <div class="flex w-full flex-grow justify-center order-1 md:order-2">
            <label class="w-full">
                <span>Current Provider Sales</span>
                <select bind:value={currentProvider} on:change={updateHistory}>
                    {#each getProviders() as item (item)}
                        <option value={item}>{capitaliseFirst(item)}</option>
                    {/each}
                </select>
            </label>
        </div>
    </div>
    <!-- <ProductList cart={$cart} data={$result} cartStore={cart} /> -->
</PageRoot>

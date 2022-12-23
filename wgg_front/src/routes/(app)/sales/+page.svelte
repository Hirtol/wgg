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
    import { updateQueryParameter } from '$lib/routing';
    import { capitaliseFirst } from '$lib/utils';
    import type { PageData } from './$types';
    import SalesList from './SalesList.svelte';

    export let data: PageData;

    let currentProvider = data.initialProvider;

    $: ({ result, cart, availableProviders } = data);

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

<PageRoot>
    <div class="grid grid-cols-1 gap-y-4 md:grid-cols-5 md:gap-4">
        <div class="flex justify-center">
            <label class="w-full">
                <span>Current Provider Sales</span>
                <select bind:value={currentProvider} on:change={updateHistory}>
                    {#each [...availableProviders.keys()] as item (item)}
                        <option value={item}>{capitaliseFirst(item)}</option>
                    {/each}
                </select>
            </label>
        </div>
        <div id="product-list-section" class="md:col-span-4">
            <SalesList {cart} data={$result.data?.proPromotions.edges.map((i) => i.node) ?? []} />
        </div>
    </div>
</PageRoot>

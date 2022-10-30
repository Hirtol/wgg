<script lang="ts">
    import { Provider } from '$lib/api/graphql_types';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { AccordionGroup, AccordionItem } from '@brainandbones/skeleton';
    import { get } from 'svelte/store';
    import type { PageData } from './$types';
    import CartOverview from './CartOverview.svelte';
    import CartProductList from './CartProductList.svelte';

    export let data: PageData;

    /**
     * Contains all providers and whether their Tally accordion is expanded.
     */
    let talliesExpanded: { [key: string]: boolean } = Object.values(Provider).reduce((result, key) => {
        const preference = get(data.preferences);
        (result as any)[key] = preference.displayPrice == key;
        return result;
    }, {});

    $: console.log(talliesExpanded);

    $: cart = data.cart
    $: preferences = data.preferences;
    $: tallies = $cart.data?.tallies ?? [];
    $: tallies.sort((a, b) => a.providerInfo.provider.localeCompare(b.providerInfo.provider));
</script>

<svelte:head>
    <title>Wgg - Cart</title>
</svelte:head>

<PageRoot>
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <CartProductList columns={['grid-cols-1', '2xl:grid-cols-2']} data={$cart.data?.contents ?? []} cart={cart} />

        <div class="card card-body h-min">
            <h4 class="text-center">Overview</h4>

            <AccordionGroup>
                {#each tallies as tally (tally.providerInfo.provider)}
                    <AccordionItem bind:open={talliesExpanded[tally.providerInfo.provider]}>
                        <svelte:fragment slot="lead">
                            <div class="flex flex-row justify-between">
                                <img
                                    src={tally.providerInfo.logoUrl}
                                    class="pointer-events-none h-8"
                                    alt={tally.providerInfo.provider} />
                            </div>
                        </svelte:fragment>

                        <h3 slot="summary">{tally.providerInfo.provider}</h3>

                        <CartOverview data={tally} slot="content" />
                    </AccordionItem>
                {/each}
            </AccordionGroup>
        </div>
    </div>
</PageRoot>

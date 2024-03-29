<script lang="ts">
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { AccordionGroup, AccordionItem } from '@skeletonlabs/skeleton';
    import type { PageData } from './$types';
    import CartOverview from './CartOverview.svelte';
    import CartProductList from './CartProductList.svelte';

    export let data: PageData;

    let { preferences, cart } = data;

    /**
     * Contains all providers and whether their Tally accordion is expanded.
     */
    let talliesExpanded: { [key: string]: boolean } = [...data.availableProviders.keys()].reduce((result, key) => {
        (result as any)[key] = $preferences.favouriteProvider == key;
        return result;
    }, {});

    $: tallies = $cart.data?.tallies ?? [];
    $: tallies.sort((a, b) => a.providerInfo.provider.localeCompare(b.providerInfo.provider));
</script>

<PageRoot>
    <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <div class="card order-2 h-min p-4 md:order-1">
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

        <CartProductList
            class="order-1 md:order-2 md:col-span-2"
            columns={['grid-cols-1', 'xl:grid-cols-2']}
            data={$cart.data?.contents ?? []}
            {cart} />
    </div>
</PageRoot>

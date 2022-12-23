<script lang="ts">
    import { PriceFilter } from '$lib/api/graphql_types';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { DisplayPriceOptionsConcrete } from '$lib/state';
    import { capitaliseFirst } from '$lib/utils';
    import { tooltip } from '@skeletonlabs/skeleton';
    import { Information } from 'carbon-icons-svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ preferences, availableProviders } = data);
    $: realAvail = [...availableProviders.keys()];
</script>

<PageRoot class="flex w-full flex-col items-center justify-center">
    <section class="w-full max-w-full space-y-2 md:max-w-md">
        <label class="w-full">
            <span
                use:tooltip={{
                    content: 'The price to display by default for aggregate ingredients',
                    color: 'text-on-primary-token',
                    regionContainer: 'max-w-fit'
                }}>
                Aggregate Display Price
                <Information class="inline" size={16} />
            </span>
            <select bind:value={$preferences.aggregateDisplayPrice}>
                {#each Object.values(PriceFilter) as item (item)}
                    <option value={item}>{capitaliseFirst(item)}</option>
                {/each}
            </select>
        </label>

        <label class="w-full">
            <span
                use:tooltip={{
                    content: 'The price to display by default in/on the cart overview.',
                    color: 'text-on-primary-token',
                    regionContainer: 'max-w-fit'
                }}>
                Cart Display Price
                <Information class="inline" size={16} />
            </span>
            <select bind:value={$preferences.displayPrice}>
                {#each DisplayPriceOptionsConcrete as item (item)}
                    <option value={item}>{capitaliseFirst(item)}</option>
                {/each}
            </select>
        </label>

        <label class="w-full">
            <span
                use:tooltip={{
                    content: 'The provider which is selected by default when entering new pages.',
                    color: 'text-on-primary-token',
                    regionContainer: 'max-w-fit'
                }}
                >Favourite Provider
                <Information class="inline" size={16} />
            </span>
            <select bind:value={$preferences.favouriteProvider}>
                {#each realAvail as item (item)}
                    <option value={item}>{capitaliseFirst(item)}</option>
                {/each}
            </select>
        </label>
    </section>
</PageRoot>

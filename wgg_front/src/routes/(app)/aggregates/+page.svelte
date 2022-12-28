<script lang="ts">
    import { GetAggregateIngredientsDocument } from '$lib/api/graphql_types';
    import { asyncQueryStore } from '$lib/api/urql';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { triggerCreateAggregateModal } from '$lib/components/product_display/aggregates';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ store, cart, preferences, client } = data);
    $: ingredientList = $store.data?.aggregateIngredients.edges.map((x) => x.node) ?? [];

    function triggerModal(): void {
        triggerCreateAggregateModal(async (resp) => {
                if (resp) {
                    ({ store } = await asyncQueryStore({
                        query: GetAggregateIngredientsDocument,
                        variables: { price: $preferences.aggregateDisplayPrice },
                        client
                    }));
                }
            })
    }
</script>

<PageRoot>
    <div class="grid grid-cols-1 gap-y-4 md:grid-cols-5 md:gap-4">
        <!-- Controls -->
        <section class="w-full">
            <button class="btn btn-filled-primary btn-sm w-full" on:click={triggerModal}
                >Create new</button>
        </section>
        <!-- Display -->
        <HeteroCardList class="md:col-span-4" data={ingredientList} cartStore={cart} />
    </div>
</PageRoot>

<script lang="ts">
    import { GetAggregateIngredientsDocument } from '$lib/api/graphql_types';
    import { asyncQueryStore } from '$lib/api/urql';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import CreateAggregateModal from '$lib/components/product_display/aggregates/CreateAggregateModal.svelte';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import { ModalComponent, ModalSettings, modalStore } from '@skeletonlabs/skeleton';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ store, cart, preferences, client } = data);
    $: ingredientList = $store.data?.aggregateIngredients.edges.map((x) => x.node) ?? [];

    function triggerCreateAggregateModal(): void {
        const modalComponent: ModalComponent = {
            ref: CreateAggregateModal,
            props: {}
        };
        const modal: ModalSettings = {
            type: 'component',
            component: modalComponent,
            response: async (resp) => {
                if (resp) {
                    ({ store } = await asyncQueryStore({
                        query: GetAggregateIngredientsDocument,
                        variables: { price: $preferences.aggregateDisplayPrice },
                        client: client
                    }));
                }
            },
            title: 'Create Aggregate Product'
        };

        modalStore.trigger(modal);
    }
</script>

<PageRoot>
    <div class="grid grid-cols-1 gap-y-4 md:grid-cols-5 md:gap-4">
        <!-- Controls -->
        <section class="w-full">
            <button class="btn btn-filled-primary btn-sm w-full" on:click={triggerCreateAggregateModal}
                >Create new</button>
        </section>
        <!-- Display -->
        <HeteroCardList class="md:col-span-4" data={ingredientList} cartStore={cart} />
    </div>
</PageRoot>

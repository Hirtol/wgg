<script lang="ts">
    import { GetAggregateIngredientsDocument } from '$lib/api/graphql_types';
    import { asyncQueryStore } from '$lib/api/urql';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import AggregateCard from '$lib/components/product_display/aggregates/AggregateCard.svelte';
    import CreateAggregateModal from '$lib/components/product_display/aggregates/CreateAggregateModal.svelte';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import { getContextPreferences } from '$lib/state';
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

    modalStore.clear();
    triggerCreateAggregateModal();
</script>

<PageRoot>
    <HeteroCardList data={ingredientList} cartStore={cart} />
</PageRoot>

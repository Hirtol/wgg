<script lang="ts">
    import PageRoot from '$lib/components/PageRoot.svelte';
    import AggregateCard from '$lib/components/product_display/aggregates/AggregateCard.svelte';
    import CreateAggregateModal from '$lib/components/product_display/aggregates/CreateAggregateModal.svelte';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import { ModalComponent, ModalSettings, modalStore } from '@skeletonlabs/skeleton';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ store, cart } = data);
    $: ingredientList = $store.data?.aggregateIngredients.edges.map((x) => x.node) ?? [];

    function triggerCreateAggregateModal(): void {
        const modalComponent: ModalComponent = {
            ref: CreateAggregateModal,
            props: {},
            slot: '<p>Help</p>'
        };
        const modal: ModalSettings = {
            type: 'component',
            component: modalComponent,
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

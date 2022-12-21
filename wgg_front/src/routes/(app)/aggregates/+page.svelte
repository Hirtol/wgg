<script lang="ts">
    import AggregateCard from '$lib/components/aggregates/AggregateCard.svelte';
import CreateAggregateModal from '$lib/components/aggregates/CreateAggregateModal.svelte';
    import { ModalComponent, ModalSettings, modalStore } from '@skeletonlabs/skeleton';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({store} = data);
    $: ingredientList = $store.data?.aggregateIngredients.edges ?? []

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

{#each ingredientList as aggregate(aggregate.cursor)}
    <AggregateCard data={aggregate.node}></AggregateCard>
{/each}

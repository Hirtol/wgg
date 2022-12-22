<!--
    @component
    The modal which allows one to edit an existing Aggregate product
-->
<script lang="ts">
    import { AggregateCardFragment, UpdateAggregateProductDocument } from '$lib/api/graphql_types';
    import { asyncMutationStore } from '$lib/api/urql';
    import { getContextPreferences } from '$lib/state';
    import { modalStore } from '@skeletonlabs/skeleton';
    import { getContextClient } from '@urql/svelte';
    import { globalLoading } from '../../global_progress/global_loading';

    /** Exposes parent props to this component. */
    export let parent: any;
    /**
     * The current aggregate ingredient.
     */
    export let aggregate: AggregateCardFragment;

    const client = getContextClient();
    const preferences = getContextPreferences();
    const formData = {
        name: aggregate.name,
        imageUrl: aggregate.imageUrl
    };

    async function onFormSubmit() {
        if (disableSubmit) return;

        let { item } = await globalLoading.submit(
            asyncMutationStore({
                query: UpdateAggregateProductDocument,
                variables: {
                    id: aggregate.id,
                    input: {
                        name: formData.name,
                        imageUrl: formData.imageUrl
                    },
                    price: $preferences.aggregateDisplayPrice
                },
                client
            })
        );

        if (!item.error) {
            if ($modalStore[0].response) $modalStore[0].response(formData);
            modalStore.close();
        }
    }

    // Base Classes
    const cBase = 'space-y-4';
    const cForm = 'border border-surface-500 p-4 space-y-4 rounded-container-token';

    $: disableSubmit = formData.name.length == 0;
</script>

<div class="modal-example-form {cBase}">
    <form id="modal-form" class="modal-form {cForm}" on:submit|preventDefault={onFormSubmit}>
        <label>
            <span>Name</span>
            <input type="text" bind:value={formData.name} placeholder="Enter name..." />
        </label>
        <label>
            <span>Image url</span>
            <input type="url" bind:value={formData.imageUrl} placeholder="Enter image url..." />
        </label>
        <input type="submit" class="hidden" aria-hidden="true" />
    </form>
    <!-- prettier-ignore -->
    <footer class="modal-footer {parent.regionFooter}">
        <button class="btn {parent.buttonNeutral}" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
        <button type="submit" form="modal-form" class="btn {parent.buttonPositive}" disabled={disableSubmit}>Submit</button>
    </footer>
</div>

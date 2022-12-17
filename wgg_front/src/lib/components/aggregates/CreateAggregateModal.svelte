<!--
    @component
    The modal which allows one to create a new Aggregate product
-->
<script lang="ts">
    import { CreateAggregateProductDocument } from '$lib/api/graphql_types';
    import { asyncMutationStore } from '$lib/api/urql';
    import { modalStore } from '@skeletonlabs/skeleton';
    import { getContextClient } from '@urql/svelte';
    import { globalLoading } from '../global_progress/global_loading';

    /** Exposes parent props to this component. */
    export let parent: any;

    const client = getContextClient();
    const formData = {
        name: ''
    };

    async function onFormSubmit() {
        let { item } = await globalLoading.submit(
            asyncMutationStore({
                query: CreateAggregateProductDocument,
                variables: {
                    input: {
                        name: formData.name,
                        ingredients: []
                    }
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
    <form class="modal-form {cForm}" on:submit|preventDefault={onFormSubmit}>
        <label>
            <span>Name</span>
            <input type="text" bind:value={formData.name} placeholder="Enter name..." />
        </label>
    </form>
    <!-- prettier-ignore -->
    <footer class="modal-footer {parent.regionFooter}">
        <button class="btn {parent.buttonNeutral}" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
        <button class="btn {parent.buttonPositive}" on:click={onFormSubmit} disabled={disableSubmit}>Submit Form</button>
    </footer>
</div>

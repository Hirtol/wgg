<!--
    @component
    The modal which allows one to add a product to an aggregate ingredient.
-->
<script lang="ts">
    import {
        AggregateCardFragment,
        GetAggregateIngredientsDocument,
        GetAggregateIngredientsModalDocument,
        ProductCardFragment,
        UpdateAggregateProductDocument
    } from '$lib/api/graphql_types';
    import { asyncMutationStore, asyncQueryStore } from '$lib/api/urql';
    import SimpleSelectableTable, { TableData, TableRow } from '$lib/components/tables/SimpleSelectableTable.svelte';
    import { getContextPreferences } from '$lib/state';
    import { modalStore } from '@skeletonlabs/skeleton';
    import { getContextClient, queryStore } from '@urql/svelte';
    import { Pause } from 'carbon-icons-svelte';
    import { triggerCreateAggregateModal } from '.';

    /** Exposes parent props to this component. */
    export let parent: any;
    /**
     * The product to add.
     */
    export let product: ProductCardFragment;

    const client = getContextClient();
    const preferences = getContextPreferences();

    /**
     * The items to update in the final mutation on 'Submit'
     */
    let toUpdate = [];
    let searchText: string = '';

    let tableStuff: TableData = {
        head: ['Name', 'Image'],
        bodyData: []
    };

    $: query = {
        query: GetAggregateIngredientsModalDocument,
        variables: {
            first: 10,
            filters: {
                hasName: searchText.length > 0 ? searchText : undefined
            },
            price: $preferences.aggregateDisplayPrice
        },
        client
    };
    $: list = queryStore(query);

    $: tableRows =
        $list.data?.aggregateIngredients.edges.map((x) => {
            return {
                index: x.cursor,
                checked:
                    x.node.ingredients.find(
                        (x) => x.id == product.id && x.providerInfo.provider == product.providerInfo.provider
                    ) != undefined,
                data: x.node
            };
        }) ?? [];
    $: tableStuff = {
        ...tableStuff,
        bodyData: tableRows
    };

    function triggerCreateModal(): void {
        triggerCreateAggregateModal(async (resp) => {
            if (resp) {
                // Refresh
                queryStore(query);
            }
        });
    }

    // Base Classes
    const cBase = 'space-y-4';
    const cForm = 'border border-surface-500 p-4 space-y-4 rounded-container-token';
</script>

<div class="modal-main {cBase}">
    <div class="flex flex-col-reverse gap-2 md:flex-row">
        <label class="w-full">
            <span>Aggregate Ingredient Search</span>
            <input type="search" placeholder="Search..." bind:value={searchText} />
        </label>
        <button
            class="btn btn-filled-accent btn-base h-min md:self-end"
            title="Create a new aggregate product"
            on:click={triggerCreateModal}>
            Create new aggregate
        </button>
    </div>

    {#if !$list.fetching && $list.data}
        {$list.data.aggregateIngredients.totalCount}
        <SimpleSelectableTable data={tableStuff} rowClass="h-8" on:selected={console.log}>
            <svelte:fragment let:item>
                <td>{item.data.name}</td>
                <td>
                    <img src={item.data.imageUrl} class="h-8" alt={item.data.name} />
                </td>
            </svelte:fragment>
        </SimpleSelectableTable>
    {:else}
        Fetching...
    {/if}

    <footer class="modal-footer {parent.regionFooter}">
        <button class="btn {parent.buttonNeutral}" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
        <button type="submit" class="btn {parent.buttonPositive}">Submit</button>
    </footer>
</div>

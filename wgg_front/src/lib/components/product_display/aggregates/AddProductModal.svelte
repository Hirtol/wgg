<!--
    @component
    The modal which allows one to add a product to an aggregate ingredient.
-->
<script lang="ts">
    import {
        AddProductModalFragment,
        AggregateFullFragment,
        AggregateUpdateChangeSet,
        GetAggregateIngredientsModalDocument,
        GetAggregateIngredientsModalQueryVariables,
        UpdateAggregateProductDocument
    } from '$lib/api/graphql_types';
    import { asyncMutationStore } from '$lib/api/urql';
    import { globalLoading } from '$lib/components/global_progress/global_loading';
    import Paginator from '$lib/components/pagination/Paginator.svelte';
    import SimpleSelectableTable, { TableData, TableRow } from '$lib/components/tables/SimpleSelectableTable.svelte';
    import { getContextPreferences } from '$lib/state';
    import { modalStore } from '@skeletonlabs/skeleton';
    import { getContextClient, queryStore } from '@urql/svelte';
    import { triggerCreateAggregateModal } from '.';

    /** Exposes parent props to this component. */
    export let parent: any;
    /**
     * The product to add.
     */
    export let product: AddProductModalFragment;

    const client = getContextClient();
    const preferences = getContextPreferences();

    /**
     * The items to update in the final mutation on 'Submit'
     */
    let toUpdate: {
        changes: AggregateUpdateChangeSet;
        source: AggregateFullFragment;
        cursor: string;
    }[] = [];
    /**
     * The current search text.
     */
    let searchText: string = '';

    let tableStuff: TableData = {
        head: ['Name', 'Image'],
        bodyData: []
    };
    let vars: GetAggregateIngredientsModalQueryVariables = {
        first: 10,
        filters: {
            hasName: searchText.length > 0 ? searchText : undefined
        },
        price: $preferences.aggregateDisplayPrice
    };
    let query = {
        query: GetAggregateIngredientsModalDocument,
        variables: vars,
        client
    };
    let list = queryStore(query);
    let paginatorSettings = {
        offset: 0,
        limit: 5
    };

    // Refresh query when updating the search text.
    $: {
        query.variables.filters.hasName = searchText.length > 0 ? searchText : undefined;

        queryStore(query);
    }
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

    async function handleSubmit() {
        let promises = toUpdate.map((x) => {
            return asyncMutationStore(
                {
                    query: UpdateAggregateProductDocument,
                    variables: {
                        id: x.source.id,
                        input: x.changes,
                        price: $preferences.aggregateDisplayPrice
                    },
                    client
                },
                undefined,
                false
            );
        });
        // Wait for all changes to resolve.
        let responses = await globalLoading.submit(Promise.all(promises));

        if ($modalStore[0].response) $modalStore[0].response(responses);
        modalStore.close();
    }

    function handleLimitChange(event: CustomEvent<number>) {
        // paginatorSettings.limit = event.detail;
        query.variables.first = paginatorSettings.limit;
        query.variables.after = undefined;
    }

    function handlePageChange(event: CustomEvent<number>) {
        console.log(event);
        paginatorSettings.offset = event.detail;

        query.variables.after =
            paginatorSettings.offset == 0
                ? undefined
                : (paginatorSettings.offset * paginatorSettings.limit - 1).toString();
    }

    /**
     * Handle an individual item being selected for inclusion
     */
    function handleSelect(event: CustomEvent<TableRow>) {
        let row = event.detail;
        let dats = row.data as AggregateFullFragment;
        let index = toUpdate.findIndex((i) => i.cursor == row.index);

        if (index !== -1) {
            toUpdate.splice(index, 1);
        } else {
            let update: AggregateUpdateChangeSet;

            if (row.checked) {
                let oldIngredients = dats.ingredients.map((x) => ({ id: x.id, provider: x.providerInfo.provider }));
                update = {
                    ingredients: oldIngredients.concat({ id: product.id, provider: product.providerInfo.provider })
                };
            } else {
                update = {
                    ingredients: dats.ingredients
                        .map((x) => ({ id: x.id, provider: x.providerInfo.provider }))
                        .filter((x) => x.id !== product.id && x.provider !== product.providerInfo.provider)
                };
            }

            toUpdate.push({
                cursor: row.index,
                source: dats,
                changes: update
            });
        }

        toUpdate = toUpdate;
    }
</script>

<div class="modal-main flex h-full min-h-full flex-col space-y-4 md:h-[60vh]">
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

    {#if $list.data}
        {$list.data.aggregateIngredients.totalCount}
        <form id="changeForm" on:submit|preventDefault={handleSubmit} class="overflow-auto overscroll-none">
            <SimpleSelectableTable data={tableStuff} rowClass="h-8" on:selected={handleSelect} withSelectAll={false}>
                <svelte:fragment let:item>
                    <td>{item.data.name}</td>
                    <td>
                        <img src={item.data.imageUrl} class="h-8" alt={item.data.name} />
                    </td>
                </svelte:fragment>
            </SimpleSelectableTable>
        </form>
        <Paginator
            bind:settings={paginatorSettings}
            on:amount={handleLimitChange}
            on:page={handlePageChange}
            amounts={[1, 2, 5, 10]}
            totalCount={$list.data?.aggregateIngredients.totalCount ?? 0} />
    {:else}
        Fetching...
    {/if}

    <footer class="modal-footer !mt-auto pt-4 {parent.regionFooter}">
        <p class="mr-auto">{toUpdate.length} Changes</p>
        <button class="btn {parent.buttonNeutral}" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
        <button type="submit" form="changeForm" class="btn {parent.buttonPositive}">Submit</button>
    </footer>
</div>

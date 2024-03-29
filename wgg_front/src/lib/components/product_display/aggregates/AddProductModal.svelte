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
        PartialFullAggregateFragment,
        UpdateAggregateProductDocument
    } from '$lib/api/graphql_types';
    import { asyncMutationStore } from '$lib/api/urql';
    import { globalLoading } from '$lib/components/global_progress/global_loading';
    import PaginatorAfter, { PaginationSettings } from '$lib/components/misc/pagination/PaginatorAfter.svelte';
    import SimpleSelectableTable, { TableData, TableRow } from '$lib/components/tables/SimpleSelectableTable.svelte';
    import { getContextPreferences, isMobileScreen } from '$lib/state';
    import { modalStore } from '@skeletonlabs/skeleton';
    import { getContextClient, queryStore, RequestPolicy } from '@urql/svelte';
    import { triggerCreateAggregateModal } from '.';

    /** Exposes parent props to this component. */
    export let parent: any;
    /**
     * The product to add.
     */
    export let product: AddProductModalFragment;

    const client = getContextClient();
    const preferences = getContextPreferences();
    const paginationLimits = [2, 5, 10];
    const tableSettings: TableData = {
        head: ['Name', 'Image']
    };

    /**
     * The items to update in the final mutation on 'Submit'
     */
    let toUpdate: {
        changes: AggregateUpdateChangeSet;
        sourceId: number;
        cursor: string;
    }[] = [];

    /**
     * The current search text.
     */
    let searchText: string = '';

    let paginatorSettings: PaginationSettings = {
        after: undefined,
        limit: paginationLimits[$isMobileScreen ? 0 : 1]
    };

    $: query = {
        query: GetAggregateIngredientsModalDocument,
        variables: {
            first: paginatorSettings.limit,
            after: paginatorSettings.after?.toString(),
            filters: {
                hasName: searchText.length > 0 ? searchText : undefined
            },
            price: $preferences.aggregateDisplayPrice
        },
        client,
        requestPolicy: 'cache-first' as RequestPolicy
    };
    $: list = queryStore(query);

    $: tableRows =
        $list.data?.aggregateIngredients.edges.map((x) => {
            return {
                index: x.cursor,
                checked: shouldBeChecked(x.node),
                data: x.node
            };
        }) ?? [];

    function shouldBeChecked(agg: PartialFullAggregateFragment): boolean {
        const alreadyOwned =
            agg.ingredients.find(
                (x) => x.id == product.id && x.providerInfo.provider == product.providerInfo.provider
            ) != undefined;

        const willUpdate =
            toUpdate
                .filter((y) => agg.id == y.sourceId)
                .find((x) =>
                    x.changes.ingredients?.find(
                        (x) => x.id == product.id && x.provider == product.providerInfo.provider
                    )
                ) != undefined;

        return alreadyOwned || willUpdate;
    }

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
                        id: x.sourceId,
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
                sourceId: dats.id,
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
        <form id="changeForm" on:submit|preventDefault={handleSubmit} class="overflow-auto overscroll-none">
            <SimpleSelectableTable
                data={tableRows}
                settings={tableSettings}
                rowClass="h-8"
                on:selected={handleSelect}
                withSelectAll={false}>
                <svelte:fragment let:item>
                    <td>{item.data.name}</td>
                    <td>
                        <img src={item.data.imageUrl} class="h-8" alt={item.data.name} />
                    </td>
                </svelte:fragment>
            </SimpleSelectableTable>
        </form>
    {:else}
        Fetching...
    {/if}

    <PaginatorAfter
        class="!align-bottom"
        bind:settings={paginatorSettings}
        amounts={paginationLimits}
        totalCount={$list.data?.aggregateIngredients.totalCount ?? 0} />

    <footer class="modal-footer !mt-auto pt-4 {parent.regionFooter}">
        <p class="mr-auto">{toUpdate.length} Changes</p>
        <button class="btn {parent.buttonNeutral}" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
        <button type="submit" form="changeForm" class="btn {parent.buttonPositive}" disabled={toUpdate.length == 0}
            >Submit</button>
    </footer>
</div>

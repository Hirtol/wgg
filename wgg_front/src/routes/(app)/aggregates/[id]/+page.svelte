<script lang="ts">
    import type { PageData } from './$types';
    import AddComponent from '$lib/components/product_display/products/AddComponent.svelte';
    import { getContextClient } from '@urql/svelte';
    import { Divider, ModalComponent, ModalSettings } from '@skeletonlabs/skeleton';
    import PriceComponent from '$lib/components/product_display/products/PriceComponent.svelte';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import ProductImage from '$lib/components/product_display/products/ProductImage.svelte';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import { Pen, TrashCan } from 'carbon-icons-svelte';
    import { asyncMutationStore } from '$lib/api/urql';
    import { DeleteAggregateIngredientDocument } from '$lib/api/graphql_types';
    import { goto } from '$app/navigation';
    import { aggregatePageRootUrl } from '$lib/routing';
    import { triggerEditAggregateModal } from '$lib/components/product_display/aggregates';
    import { triggerModal } from '$lib/components/modals';

    export let data: PageData;

    let client = getContextClient();

    $: ({ store, cart } = data);
    $: aggregate = $store.data?.aggregateIngredient;

    $: quantity = aggregate ? $cart.getAggregateQuantity(aggregate.id)?.quantity ?? 0 : 0;

    async function updateCartContent(aggregateId: number, newQuantity: number) {
        await cart.setCartContent({ __typename: 'Aggregate', aggregateId, quantity: newQuantity }, client);
    }

    function triggerEditModal() {
        if (aggregate == undefined) return;

        triggerEditAggregateModal(aggregate);
    }

    function triggerConfirmDeleteModal() {
        if (aggregate == undefined) return;

        const modal: ModalSettings = {
            type: 'confirm',
            title: 'Please confirm deletion',
            body: `Are you sure you wish to delete ${aggregate.name}?`,
            response: async (response: boolean) => {
                if (!aggregate || !response) return;
                // For some reason a single `goto` call doesn't work... TODO: Remove when no longer necessary
                goto(aggregatePageRootUrl);
                await goto(aggregatePageRootUrl);

                let _ = await asyncMutationStore({
                    query: DeleteAggregateIngredientDocument,
                    variables: { id: aggregate.id },
                    client
                });
            }
        };

        triggerModal(modal);
    }
</script>

{#if aggregate}
    <PageRoot class="px-0 pt-0">
        <section class="card">
            <header class="grid grid-cols-1 px-4 md:grid-cols-2">
                <ProductImage
                    class="h-32 w-full md:h-64"
                    data={{ imageUrl: aggregate.imageUrl, name: aggregate.name }} />

                <div class="flex flex-col justify-center gap-1">
                    <h2>{aggregate.name}</h2>

                    <PriceComponent data={{ displayPrice: aggregate.price, originalPrice: aggregate.price }} />

                    <div class="flex flex-row gap-2">
                        <AddComponent
                            class="min-h-[2.5rem] max-w-[8rem] pt-2"
                            normalButton
                            {quantity}
                            on:setQuantity={(e) => aggregate && updateCartContent(aggregate.id, e.detail)} />
                        <div class="pt-2">
                            <button
                                class="btn btn-filled-primary btn-sm !h-[2rem] py-0"
                                title="Edit aggregate ingredient"
                                on:click={triggerEditModal}>
                                <Pen size={24} />
                            </button>
                            <button
                                class="btn btn-filled-primary btn-sm !h-[2rem] py-0"
                                title="Delete aggregate ingredient"
                                on:click={triggerConfirmDeleteModal}>
                                <TrashCan size={24} />
                            </button>
                        </div>
                    </div>
                </div>
            </header>

            <Divider class="mt-4 mb-2" />

            <div class="mx-auto">
                {#if aggregate.ingredients.length > 0}
                    <HeteroCardList data={aggregate.ingredients} cartStore={cart} />
                {:else}
                    <h6 class="text-center">No ingredients yet</h6>
                {/if}
            </div>
        </section>
    </PageRoot>
{:else}
    <main class="text-center">
        <h6>Failed to gather product information, please refresh the page.</h6>
    </main>
{/if}

<script lang="ts">
    import type { PageData } from './$types';
    import { FullProductFragment, Provider } from '$lib/api/graphql_types';
    import AddComponent from '$lib/components/product_display/products/AddComponent.svelte';
    import { getContextClient } from '@urql/svelte';
    import {
        AccordionGroup,
        Divider,
        ModalComponent,
        ModalSettings,
        modalStore,
        tooltip
    } from '@skeletonlabs/skeleton';
    import ImageCarousal from '$lib/components/ImageCarousal.svelte';
    import PriceComponent from '$lib/components/product_display/products/PriceComponent.svelte';
    import ShortDecorator from './ShortDecorator.svelte';
    import PriceQuantityComponent from '$lib/components/product_display/products/PriceQuantityComponent.svelte';
    import ExtendedDescription from './ExtendedDescription.svelte';
    import SaleLabel from '$lib/components/product_display/products/SaleLabel.svelte';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { Add } from 'carbon-icons-svelte';
    import AddProductModal from '$lib/components/product_display/aggregates/AddProductModal.svelte';
    import { error } from '@sveltejs/kit';

    export let data: PageData;

    const client = getContextClient();
    let product: FullProductFragment;

    $: ({ store, cart } = data);
    $: {
        let prod = $store.data?.proProduct;

        if (prod == undefined) {
            throw error(404, { code: '404', message: 'Product does not exist' });
        } else {
            product = prod;
        }
    }
    $: quantity =
        $cart.getProductQuantity(product.providerInfo.provider, product.id).find((x) => x.origin === 'Direct')
            ?.quantity ?? 0;
    $: aggregateQuantities =
        $cart.getProductQuantity(product.providerInfo.provider, product.id).filter((x) => x.origin === 'Indirect') ??
        [];
    $: aggregates = product?.appInfo.associatedAggregates ?? [];
    $: images = product.imageUrls.map((url, i) => ({
        id: i,
        imgurl: url
    }));

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        await cart.setCartContent({ productId, provider, quantity: newQuantity, __typename: 'RawProduct' }, client);
    }

    async function updateCartContentAggregate(aggregateId: number, newQuantity: number) {
        await cart.setCartContent({ __typename: 'Aggregate', aggregateId: aggregateId, quantity: newQuantity }, client);
    }

    function triggerAddProductToAggregateModal() {
        const modalComponent: ModalComponent = {
            ref: AddProductModal,
            props: {
                product
            }
        };
        const modal: ModalSettings = {
            type: 'component',
            component: modalComponent,
            title: 'Add Product to Aggregate'
        };

        modalStore.trigger(modal);
    }
    let once = false;
    $: {
        if (product && !once) {
            modalStore.clear();
            triggerAddProductToAggregateModal();
            once = true;
        }
    }
</script>

{#if product}
    <PageRoot class="px-0 pt-0">
        <section class="card">
            <header class="grid grid-cols-1 px-4 md:grid-cols-2">
                <ImageCarousal class="p-4" {images} showThumbs={false} />

                <div class="flex flex-col justify-center gap-1">
                    <h2>{product.name}</h2>

                    <ShortDecorator
                        class="text-base text-gray-500 line-clamp-1 dark:text-gray-400"
                        data={product.decorators} />

                    <PriceQuantityComponent class="!text-base !font-normal" data={product} />

                    <PriceComponent data={product.priceInfo} />

                    <div class="flex flex-row gap-2">
                        <AddComponent
                            class="min-h-[2.5rem] max-w-[8rem] pt-2"
                            normalButton
                            {quantity}
                            on:setQuantity={(e) =>
                                updateCartContent(product.id, product.providerInfo.provider, e.detail)} />

                        {#each aggregates as agg, i (agg.id)}
                            <AddComponent
                                class="min-h-[2.5rem] max-w-[8rem] pt-2"
                                productTitle={agg.name}
                                normalButton
                                quantity={aggregateQuantities[i].quantity ?? 0}
                                on:setQuantity={(e) => updateCartContentAggregate(agg.id, e.detail)} />
                        {/each}

                        <button
                            class="btn btn-filled-accent btn-sm ml-auto"
                            title="Add to aggregate product"
                            on:click={triggerAddProductToAggregateModal}>
                            <Add size={24} />
                            Add to aggregate
                        </button>
                    </div>

                    {#if product.saleInformation}
                        <SaleLabel
                            textSize="!text-base"
                            text={product.saleInformation.label}
                            saleType={product.saleInformation.saleType} />
                    {/if}

                    {#if product.unavailableDetails}
                        <span
                            use:tooltip={{
                                content:
                                    product.unavailableDetails.explanationLong ??
                                    product.unavailableDetails.explanationShort ??
                                    '',
                                background: '!bg-accent-500',
                                regionContainer: 'max-w-fit'
                            }}
                            class="badge bg-warning-400 !text-base dark:bg-warning-700">
                            Unavailable: {product.unavailableDetails.explanationShort}
                        </span>
                    {/if}
                </div>
            </header>

            <Divider class="mt-4 mb-2" />

            <div class="mx-auto px-4">
                <ExtendedDescription {product} />
            </div>
        </section>
    </PageRoot>
{:else}
    <main class="text-center">
        <h6>Failed to gather product information, please refresh the page.</h6>
    </main>
{/if}

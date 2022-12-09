<script lang="ts">
    import type { PageData } from './$types';
    import { Provider, TextType } from '$lib/api/graphql_types';
    import AddComponent from '$lib/components/products/AddComponent.svelte';
    import { getContextClient } from '@urql/svelte';
    import { AccordionGroup, Divider, tooltip } from '@brainandbones/skeleton';
    import ImageCarousal from '$lib/components/ImageCarousal.svelte';
    import PriceComponent from '$lib/components/products/PriceComponent.svelte';
    import ShortDecorator from './ShortDecorator.svelte';
    import PriceQuantityComponent from '$lib/components/products/PriceQuantityComponent.svelte';
    import ExtendedDescription from './ExtendedDescription.svelte';
    import SaleLabel from '$lib/components/products/SaleLabel.svelte';

    export let data: PageData;

    let client = getContextClient();

    $: ({ store, cart } = data);
    $: product = $store.data?.proProduct;

    $: images =
        product?.imageUrls.map((url, i) => ({
            id: i,
            imgurl: url
        })) ?? [];

    $: quantity = product ? $cart.getProductQuantity(product.providerInfo.provider, product.id)[0]?.quantity ?? 0 : 0;

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        await cart.setCartContent({ productId, provider, quantity: newQuantity, __typename: 'RawProduct' }, client);
    }
</script>

{#if product}
    <main class="container mx-auto">
        <section class="card">
            <header class="grid grid-cols-1 px-4 md:grid-cols-2">
                <ImageCarousal class="card-body" {images} showThumbs={false} />

                <div class="flex flex-col justify-center gap-1">
                    <h2>{product.name}</h2>

                    <ShortDecorator
                        class="text-base text-gray-500 line-clamp-1 dark:text-gray-400"
                        data={product.decorators} />

                    <PriceQuantityComponent class="!text-base !font-normal" data={product} />

                    <PriceComponent data={product.priceInfo} />

                    <AddComponent
                        class="min-h-[2.5rem] max-w-[8rem] pt-2"
                        normalButton
                        {quantity}
                        on:setQuantity={(e) =>
                            product && updateCartContent(product.id, product.providerInfo.provider, e.detail)} />

                    {#if product.saleInformation}
                        <SaleLabel text={product.saleInformation.label} saleType={product.saleInformation.saleType}/>
                    {/if}

                    {#if product.unavailableDetails}
                        <span
                            use:tooltip={{
                                content: product.unavailableDetails.explanationLong ?? product.unavailableDetails.explanationShort ?? '',
                                background: '!bg-accent-500',
                                regionContainer: 'max-w-fit'
                            }}
                            class="badge bg-warning-400 !text-lg dark:bg-warning-700">
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
    </main>
{:else}
    <main class="text-center">
        <h6>Failed to gather product information, please refresh the page.</h6>
    </main>
{/if}

<style>
</style>
